use bytecodec::bytes::BytesEncoder;
use bytecodec::null::NullDecoder;
use fibers_http_server::{HandleRequest, Reply, Req, Res, Result, ServerBuilder, Status};
use futures::future::ok;
use httpcodec::{BodyDecoder, BodyEncoder};

#[derive(Debug, Clone)]
pub struct FileServer;
impl FileServer {
    pub fn new() -> Self {
        FileServer
    }
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

    fn handle_request(&self, _req: Req<Self::ReqBody>) -> Self::Reply {
        println!("# REQ: {:?}", _req);
        Box::new(ok(Res::new(Status::Ok, Vec::new())))
    }
}
