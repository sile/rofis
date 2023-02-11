use std::io::{Read, Write};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Head,
    Get,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: Url,
}

impl HttpRequest {
    pub fn from_reader<R: Read>(reader: &mut R) -> orfail::Result<Result<Self, HttpResponse>> {
        orfail::todo!()
    }
}

pub struct HttpResponse {
    pub status: &'static str,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn bad_request(reason: String) -> Self {
        todo!()
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
