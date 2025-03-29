use orfail::OrFail;
use rofis::{
    dirs_index::DirsIndex,
    http::{HttpMethod, HttpRequest, HttpResponse, HttpResponseBody},
};
use std::{
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct Args {
    port: u16,
    root_dir: PathBuf,
    log_level: log::LevelFilter,
    daemonize: bool,
    version: Option<String>,
    help: Option<String>,
}

impl Args {
    fn parse() -> noargs::Result<Self> {
        let mut args = noargs::raw_args();

        args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
        args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");
        if noargs::HELP_FLAG.take(&mut args).is_present() {
            args.metadata_mut().help_mode = true;
        }

        Ok(Self {
            port: noargs::opt("port")
                .short('p')
                .ty("INTEGER")
                .default("8080")
                .doc("Listen port number.")
                .take(&mut args)
                .parse()?,
            root_dir: noargs::opt("root-dir")
                .short('r')
                .ty("PATH")
                .env("PWD")
                .doc("Root directory.")
                .take(&mut args)
                .parse()?,
            log_level: noargs::opt("log-level")
                .short('l')
                .ty("DEBUG | INFO | WARN | ERROR")
                .default("INFO")
                .doc("Log level.")
                .take(&mut args)
                .parse()?,
            daemonize: noargs::flag("daemonize")
                .short('d')
                .doc("Daemonize HTTP server.")
                .take(&mut args)
                .is_present(),
            version: noargs::VERSION_FLAG.take(&mut args).is_present().then(|| {
                format!(
                    "{} {}\n",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_DESCRIPTION")
                )
            }),
            help: args.finish()?,
        })
    }
}

fn main() -> noargs::Result<()> {
    let args = Args::parse()?;
    if let Some(text) = args.version.as_ref().or(args.help.as_ref()) {
        print!("{text}");
        return Ok(());
    }

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

    let root_dir = args.root_dir;
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
    let Ok(content) = std::fs::read(&path) else {
        return HttpResponse::internal_server_error();
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let body = HttpResponseBody::Content(content);
    HttpResponse::ok(mime, body)
}

fn head_file<P: AsRef<Path>>(path: P) -> HttpResponse {
    let Ok(content) = std::fs::read(&path) else {
        return HttpResponse::internal_server_error();
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let body = HttpResponseBody::Length(content.len());
    HttpResponse::ok(mime, body)
}

fn write_response(mut socket: TcpStream, response: HttpResponse) {
    if let Err(e) = response.to_writer(&mut socket) {
        log::warn!("Failed to write HTTP response: {e}");
    } else {
        log::info!("Wrote: {response:?}");
    }
}
