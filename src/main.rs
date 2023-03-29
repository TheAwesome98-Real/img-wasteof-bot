#![feature(const_option_ext)]

use rust_socketio::{asynchronous::ClientBuilder, Payload};

const VERSION: &str = option_env!("CARGO_PKG_VERSION").unwrap_or("(unknown)");
const SERVER: &str = "https://api.wasteof.money";

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting img v{}", VERSION);
    log::info!("connecting to server '{}'...", SERVER);
    match ClientBuilder::new(SERVER)
        .on("updateMessageCount", |payload, _client| match payload {
            Payload::String(string) => Box::pin(async move {
                log::info!("Received: {}", string);
            }),
            data => Box::pin(async move {
                log::warn!("can't handle data: {:#?}", data);
            }),
        })
        .on("error", |err, _client| {
            Box::pin(async move {
                log::error!("Error: {:#?}", err);
            })
        })
        .on("close", |_payload, _client| {
            Box::pin(async move {
                log::info!("disconnected from server");
            })
        })
        .on("message", |payload, _| {
            Box::pin(async move {
                log::info!("other message: {payload:#?}");
            })
        })
        .connect()
        .await
    {
        Ok(_client) => log::info!("connected!"),
        Err(err) => {
            log::error!("failed to connect to the server: {err}");
            log::error!("raw error is as follows:\n {err:#?}")
        }
    }
}
