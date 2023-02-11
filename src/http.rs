use mime_guess::Mime;
use orfail::OrFail;
use std::io::{BufRead, BufReader, Read, Write};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Head,
    Get,
}

#[derive(Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    url: Url,
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
        } else {
            return Ok(Err(HttpResponse::method_not_allowed()));
        };

        // Path.
        if !line.starts_with("/") {
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

        Ok(Ok(Self { method, url }))
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn path(&self) -> &str {
        self.url.path()
    }
}

pub struct HttpResponse {
    pub status: &'static str,
    pub header: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn bad_request() -> Self {
        Self {
            status: "400 Bad Request",
            header: Vec::new(),
            body: b"Bad Request".to_vec(),
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: "404 Not Found",
            header: Vec::new(),
            body: b"Not Found".to_vec(),
        }
    }

    pub fn method_not_allowed() -> Self {
        Self {
            status: "405 Method Not Allowed",
            header: vec![("Allow", "GET, HEAD".to_owned())],
            body: b"Method Not Allowed".to_vec(),
        }
    }

    pub fn multiple_choices() -> Self {
        Self {
            status: "303 Multiple Choices",
            header: Vec::new(),
            body: b"Multiple Choices".to_vec(),
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            status: "500 Internal Server Error",
            header: Vec::new(),
            body: b"Internal Server Error".to_vec(),
        }
    }

    pub fn ok(mime: Mime, body: Vec<u8>) -> Self {
        Self {
            status: "200 OK",
            header: vec![("Content-Type", mime.to_string())],
            body,
        }
    }

    pub fn to_writer<W: Write>(&self, writer: &mut W) -> orfail::Result<()> {
        write!(writer, "HTTP/1.1 {}\r\n", self.status).or_fail()?;
        write!(writer, "Content-Length: {}\r\n", self.body.len()).or_fail()?;
        write!(writer, "Connection: close\r\n").or_fail()?;
        for (name, value) in &self.header {
            write!(writer, "{}: {}\r\n", name, value).or_fail()?;
        }
        write!(writer, "\r\n").or_fail()?;
        writer.write_all(&self.body).or_fail()?;
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
