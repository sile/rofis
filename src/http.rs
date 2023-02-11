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

    pub fn method_not_allowed() -> Self {
        Self {
            status: "405 Method Not Allowed",
            header: vec![("Allow", "GET, HEAD".to_owned())],
            body: b"Method Not Allowed".to_vec(),
        }
    }

    pub fn to_writer<W: Write>(&self, writer: &mut W) -> orfail::Result<()> {
        orfail::todo!()
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
