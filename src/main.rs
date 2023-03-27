#![feature(const_option_ext)]

use rust_socketio::{ClientBuilder, Payload, RawClient};

const VERSION: &str = option_env!("CARGO_PKG_VERSION").unwrap_or("(unknown)");

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting img v{}", VERSION);
    log::info!("connecting to server...");
    match ClientBuilder::new("http://wasteof.money")
        .on("updateMessageCount", |payload: Payload, _socket: RawClient| {
            match payload {
                Payload::String(string) => log::info!("Received: {}", string),
                data => log::warn!("can't handle data: {:#?}", data),
            }
        })
        .on("error", |err, _| log::error!("Error: {:#?}", err))
        .connect() {
            Ok(_client) => log::info!("connected!"),
            Err(err) => log::error!("failed to connect to the server: {err}: {err:?}"),
        }
}
