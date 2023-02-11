use clap::Parser;
use orfail::OrFail;
use rofis::{
    dirs_index::DirsIndex,
    http::{HttpMethod, HttpRequest, HttpResponse, HttpResponseBody},
};
use std::net::TcpListener;

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    // TODO: root, port
}

fn main() -> orfail::Result<()> {
    env_logger::try_init().or_fail()?;
    let _args = Args::parse();

    let root_dir = std::env::current_dir().or_fail()?;
    log::info!("Starts building directories index: root_dir={root_dir:?}");
    let mut dirs_index = DirsIndex::build(&root_dir).or_fail()?;
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
                let response = handle_request(&mut dirs_index, &request);
                if response.is_not_found() {
                    // The directories index may be outdated.
                    log::info!("Starts re-building directories index: root_dir={root_dir:?}");
                    dirs_index = DirsIndex::build(&root_dir).or_fail()?;
                    log::info!(
                        "Finished re-building directories index: entries={}",
                        dirs_index.len()
                    );
                    handle_request(&mut dirs_index, &request)
                } else {
                    response
                }
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

fn handle_request(dirs_index: &mut DirsIndex, request: &HttpRequest) -> HttpResponse {
    // TODO: handle _WATCH
    let path = request.path();
    let (dir, name) = if path.ends_with('/') {
        (path, "index.html")
    } else {
        let mut iter = path.rsplitn(2, '/');
        let Some(name) = iter.next() else {
            return HttpResponse::bad_request();
        };
        let Some(dir) = iter.next() else {
            return HttpResponse::bad_request();
        };
        (dir, name)
    };

    let candidates = dirs_index
        .find_dirs_by_suffix(dir.trim_matches('/'))
        .into_iter()
        .map(|dir| dir.join(name))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return HttpResponse::not_found();
    } else if candidates.len() > 1 {
        return HttpResponse::multiple_choices(candidates.len());
    }

    let Ok(content) = std::fs::read(&candidates[0]) else  {
         return HttpResponse::internal_server_error();
    };
    let mime = mime_guess::from_path(name).first_or_octet_stream();
    let body = match request.method() {
        HttpMethod::Head => HttpResponseBody::Length(content.len()),
        HttpMethod::Get => HttpResponseBody::Content(content),
    };
    HttpResponse::ok(mime, body)
}
