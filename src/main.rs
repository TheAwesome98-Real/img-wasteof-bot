#![feature(const_option_ext)]

use rust_socketio::{asynchronous::ClientBuilder, Payload};
use serde::Deserialize;
use serde_json::json;

const VERSION: &str = option_env!("CARGO_PKG_VERSION").unwrap_or("(unknown)");
const SERVER: &str = "https://api.wasteof.money";

mod cfg {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Authentication {
        pub username: String,
        pub password: String,
    }

    #[derive(Deserialize)]
    pub struct Configuration {
        pub authentication: Authentication,
    }
}

#[derive(Deserialize)]
struct Token {
    token: String,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting img v{}", VERSION);
    log::info!("loading configuration...");
    let config_string = match std::fs::read_to_string("Img.toml") {
        Ok(string) => string,
        Err(err) => {
            log::error!("could not load configuration: {err}");
            std::process::exit(1);
        }
    };
    let config: cfg::Configuration = match toml::from_str(&config_string) {
        Ok(cfg) => cfg,
        Err(err) => {
            log::error!("could not parse configuration: {err}");
            std::process::exit(1);
        }
    };
    log::info!("authenticating...");
    let body = serde_json::json!({
        "username": config.authentication.username,
        "password": config.authentication.password
    });
    let token = match reqwest::Client::default()
        .post(format!("{SERVER}/session"))
        .json(&body)
        .send()
        .await
    {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str::<Token>(&text) {
                Ok(token) => token.token,
                Err(err) => {
                    log::error!("could not parse token: {err}");
                    std::process::exit(1);
                }
            },
            Err(err) => {
                log::error!("could not parse text: {err}");
                std::process::exit(1);
            }
        },
        Err(err) => {
            log::error!("could not post /session: {err}");
            std::process::exit(1);
        }
    };
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
                match err {
                    Payload::Binary(_) => unreachable!("errors aren't binary i think"),
                    Payload::String(err) => log::warn!("socket error: {err}"),
                }
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
            log::error!("raw error is as follows:\n {err:#?}");
            std::process::exit(1);
        }
    }
    loop {
        let bio = format!(
            "@lily's wasteof.money image editor bot (last ping at {})",
            chrono::Local::now()
        );
        let body = json!({ "bio": bio });
        log::info!("updating bio with current time");
        match reqwest::Client::default()
            .put(format!(
                "{SERVER}/users/{}/bio",
                config.authentication.username
            ))
            .header("Authorization", token.clone())
            .json(&body)
            .send()
            .await
        {
            Ok(_) => log::info!("updated bio"),
            Err(err) => {
                log::warn!("failed to update bio: {err}");
                log::warn!("users may think the bot is offline!");
            }
        };
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
