use std::net::TcpListener;

use clap::Parser;
use orfail::OrFail;
use rofis::{dirs_index::DirsIndex, http::HttpRequest};

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    // TODO: root, port
}

fn main() -> orfail::Result<()> {
    env_logger::try_init().or_fail()?;
    let _args = Args::parse();

    log::info!("Starts building directories index");
    let dirs_index = DirsIndex::build(std::env::current_dir().or_fail()?).or_fail()?;
    log::info!(
        "Finished building directories index: entries={}",
        dirs_index.len()
    );

    let port = 8080;
    let listener = TcpListener::bind(("127.0.0.1", port)).or_fail()?;
    log::info!("Started HTTP server on {port} port");

    for socket in listener.incoming() {
        let mut socket = match socket {
            Ok(socket) => socket,
            Err(e) => {
                log::warn!("Failed to accept socket: {e}");
                continue;
            }
        };
        log::info!("Accepted a new client: {socket:?}");

        let request = match HttpRequest::from_reader(&mut socket) {
            Ok(request) => request,
            Err(e) => {
                log::warn!("Failed to read HTTP request: {e}");
                orfail::todo!(); // TODO: return BadRequest
            }
        };
    }

    Ok(())
}
