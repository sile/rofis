use clap::Parser;
use orfail::OrFail;
use rofis::{
    dirs_index::DirsIndex,
    http::{HttpMethod, HttpRequest, HttpResponse, HttpResponseBody},
};
use std::{
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

/// Read-only HTTP file server.
#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    /// Listen port number.
    #[clap(short, long, default_value_t = 8080)]
    port: u16,

    /// Root directory [default: $PWD].
    #[clap(short, long)]
    root_dir: Option<PathBuf>,

    /// Log level (DEBUG | INFO | WARN | ERROR)..
    #[clap(short, long, default_value_t = log::LevelFilter::Info)]
    log_level: log::LevelFilter,

    /// Daemonize HTTP server.
    #[clap(short, long)]
    daemonize: bool,
}

fn main() -> orfail::Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(args.log_level)
        .try_init()
        .or_fail()?;

    if args.daemonize {
        daemonize::Daemonize::new()
            .working_directory(std::env::current_dir().or_fail()?)
            .start()
            .or_fail()?;
    }

    let root_dir = args
        .root_dir
        .clone()
        .unwrap_or(std::env::current_dir().or_fail()?);
    log::info!("Starts building directories index: root_dir={root_dir:?}");
    let mut dirs_index = DirsIndex::build(&root_dir).or_fail()?;
    log::info!(
        "Finished building directories index: entries={}",
        dirs_index.len()
    );

    let listener = TcpListener::bind(("127.0.0.1", args.port)).or_fail()?;
    log::info!("Started HTTP server on {} port", args.port);

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
                let result = resolve_path(&dirs_index, &request).or_else(|response| {
                    if response.is_not_found() {
                        // The directories index may be outdated.
                        log::info!(
                            "Starts re-building directories index: root_dir={:?}",
                            dirs_index.root_dir()
                        );
                        match DirsIndex::build(dirs_index.root_dir()).or_fail() {
                            Ok(new_dirs_index) => {
                                dirs_index = new_dirs_index;
                                log::info!(
                                    "Finished re-building directories index: entries={}",
                                    dirs_index.len()
                                );
                                resolve_path(&dirs_index, &request)
                            }
                            Err(e) => {
                                let e: orfail::Failure = e;
                                log::warn!("Failed to re-build directories index: {e}");
                                Err(response)
                            }
                        }
                    } else {
                        Err(response)
                    }
                });

                match result {
                    Err(response) => response,
                    Ok(resolved_path) => match request.method() {
                        HttpMethod::Head => head_file(resolved_path),
                        HttpMethod::Get => get_file(resolved_path),
                        HttpMethod::Watch => {
                            watch_file(resolved_path, socket);
                            continue;
                        }
                    },
                }
            }
            Ok(Err(response)) => response,
            Err(e) => {
                let e: orfail::Failure = e;
                log::warn!("Failed to read HTTP request: {e}");
                continue;
            }
        };
        write_response(socket, response);
    }

    Ok(())
}

fn resolve_path(dirs_index: &DirsIndex, request: &HttpRequest) -> Result<PathBuf, HttpResponse> {
    let path = request.path();
    let (dir, name) = if path.ends_with('/') {
        (path, "index.html")
    } else {
        let mut iter = path.rsplitn(2, '/');
        let Some(name) = iter.next() else {
            return Err(HttpResponse::bad_request());
        };
        let Some(dir) = iter.next() else {
            return Err(HttpResponse::bad_request());
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
        return Err(HttpResponse::not_found());
    } else if candidates.len() > 1 {
        return Err(HttpResponse::multiple_choices(candidates.len()));
    }
    Ok(candidates[0].clone())
}

fn get_file<P: AsRef<Path>>(path: P) -> HttpResponse {
    let Ok(content) = std::fs::read(&path) else  {
         return HttpResponse::internal_server_error();
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let body = HttpResponseBody::Content(content);
    HttpResponse::ok(mime, body)
}

fn head_file<P: AsRef<Path>>(path: P) -> HttpResponse {
    let Ok(content) = std::fs::read(&path) else  {
         return HttpResponse::internal_server_error();
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let body = HttpResponseBody::Length(content.len());
    HttpResponse::ok(mime, body)
}

fn watch_file(path: PathBuf, socket: TcpStream) {
    let Some(mtime) = get_mtime(&path) else {
        write_response(socket, HttpResponse::internal_server_error());
        return;
    };
    log::info!("Starts watching: path={path:?}, mtime={mtime:?}");

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(100));
        let latest_mtime = get_mtime(&path);
        if latest_mtime != Some(mtime) {
            log::info!("Stopped watching: path={path:?}, mtime={latest_mtime:?}");
            let response = get_file(path);
            write_response(socket, response);
            return;
        }
    });
}

fn get_mtime<P: AsRef<Path>>(path: P) -> Option<SystemTime> {
    path.as_ref().metadata().ok()?.modified().ok()
}

fn write_response(mut socket: TcpStream, response: HttpResponse) {
    if let Err(e) = response.to_writer(&mut socket) {
        log::warn!("Failed to write HTTP response: {e}");
    } else {
        log::info!("Wrote: {response:?}");
    }
}
