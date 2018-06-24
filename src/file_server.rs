use bytecodec::bytes::BytesEncoder;
use bytecodec::null::NullDecoder;
use fibers_http_server::{HandleRequest, Reply, Req, Res, Result, ServerBuilder, Status};
use futures::future::ok;
use httpcodec::{BodyDecoder, BodyEncoder, HeaderField};
use mime_guess;
use slog::Logger;
use std::fs::File;
use std::io::Read;

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
        info!(self.0.logger, "Request: path={}", req.url().path());
        let path = format!(".{}", req.url().path());
        let result = File::open(&path).and_then(|mut f| {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            Ok(buf)
        });
        match result {
            Err(e) => Box::new(ok(Res::new(
                Status::InternalServerError,
                e.to_string().into_bytes(),
            ))),
            Ok(content) => {
                let mut res = Res::new(Status::Ok, content);
                let mime = mime_guess::guess_mime_type(&path);
                res.header_mut()
                    .add_field(HeaderField::new("Content-Type", &mime.to_string()).expect("TODO"));
                Box::new(ok(res))
            }
        }
    }
}
