#[macro_use]
extern crate clap;
extern crate fibers;
extern crate fibers_http_server;
extern crate futures;
extern crate rofis;
extern crate slog;
extern crate sloggers;
#[macro_use]
extern crate trackable;

use clap::Arg;
use fibers::{Executor, Spawn, ThreadPoolExecutor};
use fibers_http_server::ServerBuilder;
use futures::Future;
use rofis::FileServer;
use sloggers::terminal::TerminalLoggerBuilder;
use sloggers::Build;

macro_rules! try_parse {
    ($expr:expr) => {
        track_try_unwrap!(track_any_err!($expr.parse()))
    };
}

fn main() {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("PORT")
                .long("port")
                .short("p")
                .takes_value(true)
                .default_value("8080"),
        )
        .arg(
            Arg::with_name("LOG_LEVEL")
                .long("log-level")
                .takes_value(true)
                .default_value("info")
                .possible_values(&["debug", "info", "warning", "error"]),
        )
        .get_matches();
    let port = matches.value_of("PORT").expect("Never fails");
    let bind_addr = try_parse!(format!("0.0.0.0:{}", port));
    let log_level = try_parse!(matches.value_of("LOG_LEVEL").expect("Never fails"));
    let logger = track_try_unwrap!(TerminalLoggerBuilder::new().level(log_level).build());

    let executor = track_try_unwrap!(track_any_err!(ThreadPoolExecutor::with_thread_count(1)));
    let mut builder = ServerBuilder::new(bind_addr);
    builder.logger(logger.clone());

    let file_server = FileServer::new(logger);
    track_try_unwrap!(file_server.register(&mut builder));

    let http_server = builder.finish(executor.handle());
    executor.spawn(http_server.map_err(|e| panic!("{}", e)));
    track_try_unwrap!(track_any_err!(executor.run()));
}
