use std::io::Read;
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
    pub fn from_reader<R: Read>(reader: &mut R) -> orfail::Result<Self> {
        todo!()
    }
}
