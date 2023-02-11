use mime_guess::Mime;
use orfail::OrFail;
use std::io::{BufRead, BufReader, Read, Write};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Head,
    Get,

    // rofis original method.
    Watch,
}

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
}

impl HttpRequest {
    pub fn from_reader<R: Read>(reader: R) -> orfail::Result<Result<Self, HttpResponse>> {
        let mut reader = BufReader::new(reader);

        let mut buf = String::new();
        reader.read_line(&mut buf).or_fail()?;
        let mut line = &buf[..];
        if !line.ends_with(" HTTP/1.1\r\n") {
            return Ok(Err(HttpResponse::bad_request()));
        }

        // Method.
        let method = if line.starts_with("GET ") {
            line = &line["GET ".len()..];
            HttpMethod::Get
        } else if line.starts_with("HEAD ") {
            line = &line["HEAD ".len()..];
            HttpMethod::Head
        } else if line.starts_with("WATCH ") {
            line = &line["WATCH ".len()..];
            HttpMethod::Watch
        } else {
            return Ok(Err(HttpResponse::method_not_allowed()));
        };

        // Path.
        if !line.starts_with('/') {
            return Ok(Err(HttpResponse::bad_request()));
        }
        let path = &line[..line.find(" HTTP/1.1\r\n").or_fail()?];
        let Ok(url) = Url::options()
            .base_url(Some(&Url::parse("http://localhost/").or_fail()?))
            .parse(path) else {
            return Ok(Err(HttpResponse::bad_request()));
        };

        // Header.
        loop {
            buf.clear();
            reader.read_line(&mut buf).or_fail()?;
            if buf == "\r\n" {
                break;
            }
        }

        Ok(Ok(Self {
            method,
            path: url.path().to_owned(),
        }))
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub struct HttpResponse {
    status: &'static str,
    header: Vec<(&'static str, String)>,
    body: HttpResponseBody,
}

impl HttpResponse {
    pub fn bad_request() -> Self {
        Self {
            status: "400 Bad Request",
            header: Vec::new(),
            body: HttpResponseBody::Content(b"Bad Request".to_vec()),
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: "404 Not Found",
            header: Vec::new(),
            body: HttpResponseBody::Content(b"Not Found".to_vec()),
        }
    }

    pub fn method_not_allowed() -> Self {
        Self {
            status: "405 Method Not Allowed",
            header: vec![("Allow", "GET, HEAD".to_owned())],
            body: HttpResponseBody::Content(b"Method Not Allowed".to_vec()),
        }
    }

    pub fn multiple_choices(candidates: usize) -> Self {
        Self {
            status: "303 Multiple Choices",
            header: Vec::new(),
            body: HttpResponseBody::Content(
                format!("Multiple Choices: {candidates} candidates").into_bytes(),
            ),
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            status: "500 Internal Server Error",
            header: Vec::new(),
            body: HttpResponseBody::Content(b"Internal Server Error".to_vec()),
        }
    }

    pub fn ok(mime: Mime, body: HttpResponseBody) -> Self {
        Self {
            status: "200 OK",
            header: vec![("Content-Type", mime.to_string())],
            body,
        }
    }

    pub fn is_not_found(&self) -> bool {
        self.status == "404 Not Found"
    }

    pub fn to_writer<W: Write>(&self, writer: &mut W) -> orfail::Result<()> {
        write!(writer, "HTTP/1.1 {}\r\n", self.status).or_fail()?;
        write!(writer, "Content-Length: {}\r\n", self.body.len()).or_fail()?;
        write!(writer, "Connection: close\r\n").or_fail()?;
        for (name, value) in &self.header {
            write!(writer, "{name}: {value}\r\n").or_fail()?;
        }
        write!(writer, "\r\n").or_fail()?;
        if let HttpResponseBody::Content(content) = &self.body {
            writer.write_all(content).or_fail()?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HttpResponse {{ status: {:?}, body: {} bytes }}",
            self.status,
            self.body.len()
        )
    }
}

#[derive(Debug)]
pub enum HttpResponseBody {
    Content(Vec<u8>),
    Length(usize),
}

impl HttpResponseBody {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        match self {
            HttpResponseBody::Content(x) => x.len(),
            HttpResponseBody::Length(x) => *x,
        }
    }
}
