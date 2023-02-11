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
        let mut socket = match socket.or_fail() {
            Ok(socket) => socket,
            Err(e) => {
                let e: orfail::Failure = e;
                log::warn!("Failed to accept socket: {e}");
                continue;
            }
        };
        log::debug!("Accepted a client socket: {socket:?}");

        let response = match HttpRequest::from_reader(&mut socket).or_fail() {
            Ok(Ok(request)) => {
                log::info!("Read: {request:?}");
                orfail::todo!();
            }
            Ok(Err(response)) => response,
            Err(e) => {
                let e: orfail::Failure = e;
                log::warn!("Failed to read HTTP request: {e}");
                continue;
            }
        };
        if let Err(e) = response.to_writer(&mut socket) {
            log::warn!("Failed to write HTTP response: {e}");
        }
        log::info!("Wrote: {response:?}");
    }

    Ok(())
}
