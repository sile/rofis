use bytecodec::bytes::BytesEncoder;
use bytecodec::null::NullDecoder;
use fibers_http_server::{HandleRequest, Reply, Req, Res, ServerBuilder, Status};
use futures::future::ok;
use httpcodec::{BodyDecoder, BodyEncoder, HeadBodyEncoder, HeaderField};
use mime_guess;
use slog::Logger;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use {Error, Result};

/// Read-only HTTP file server.
#[derive(Debug, Clone)]
pub struct FileServer {
    logger: Logger,
}
impl FileServer {
    /// Makes a new `FileServer` instance.
    pub fn new(logger: Logger) -> Self {
        FileServer { logger }
    }

    /// Registers this server to the given HTTP server builder.
    pub fn register(self, builder: &mut ServerBuilder) -> Result<()> {
        track!(builder.add_handler(Get(self.clone())))?;
        track!(builder.add_handler(Head(self.clone())))?;
        Ok(())
    }
}

struct Get(FileServer);
impl HandleRequest for Get {
    const METHOD: &'static str = "GET";
    const PATH: &'static str = "/**";

    type ReqBody = ();
    type ResBody = Vec<u8>;
    type Decoder = BodyDecoder<NullDecoder>;
    type Encoder = BodyEncoder<BytesEncoder<Vec<u8>>>;
    type Reply = Reply<Self::ResBody>;

    fn handle_request(&self, req: Req<Self::ReqBody>) -> Self::Reply {
        info!(self.0.logger, "GET: path={}", req.url().path());
        Box::new(ok(handle_get(req)))
    }
}

struct Head(FileServer);
impl HandleRequest for Head {
    const METHOD: &'static str = "HEAD";
    const PATH: &'static str = "/**";

    type ReqBody = ();
    type ResBody = Vec<u8>;
    type Decoder = BodyDecoder<NullDecoder>;
    type Encoder = HeadBodyEncoder<BodyEncoder<BytesEncoder<Vec<u8>>>>;
    type Reply = Reply<Self::ResBody>;

    fn handle_request(&self, req: Req<Self::ReqBody>) -> Self::Reply {
        info!(self.0.logger, "HEAD: path={}", req.url().path());
        Box::new(ok(handle_get(req)))
    }
}

fn handle_get(req: Req<()>) -> Res<Vec<u8>> {
    let mut path = format!(".{}", req.url().path());
    if path.ends_with('/') {
        path.push_str("index.html");
    }
    let result = track!(read_file(&path); path);
    match result {
        Err(e) => {
            let status = e.to_http_status();
            Res::new(status, e.to_string().into_bytes())
        }
        Ok(content) => {
            let mut res = Res::new(Status::Ok, content);
            add_mime(&mut res, &path);
            res
        }
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut file = track!(File::open(&path).map_err(Error::from))?;
    let mut buf = Vec::new();
    track!(file.read_to_end(&mut buf).map_err(Error::from))?;
    Ok(buf)
}

fn add_mime(res: &mut Res<Vec<u8>>, path: &str) {
    let mime = mime_guess::guess_mime_type(&path).to_string();
    let field = HeaderField::new("Content-Type", &mime).expect("Never fails");
    res.header_mut().add_field(field);
}
