use actix_web::{web, App, HttpServer};
use flexi_logger::Logger;
use futures::stream::StreamExt;
use log::info;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::reqwest::Url;
use matrix_sdk::Client;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use std::error::Error;
use std::process;
use tokio::sync::Mutex;

use threematrix::matrix::on_stripped_state_member;
use threematrix::threema::ThreemaClient;
use threematrix::{
    matrix_incoming_message_handler, threema_incoming_message_handler, AppState, LoggerConfig,
    ThreematrixConfig,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut logger = Logger::try_with_str(format!("{}=info", CRATE_NAME))?.start()?;

    let cfg = ThreematrixConfig::new("./threematrix_cfg.toml");
    info!(
        "Starting Threematrix Server v{}. Waiting for Threema callback on {}:{}",
        VERSION,
        cfg.threema.host.clone().unwrap_or("localhost".to_owned()),
        cfg.threema.port.clone().unwrap_or(443)
    );

    let mut signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT])?;

    if let Some(LoggerConfig { level }) = cfg.logger {
        logger.parse_new_spec(format!("{}={}", CRATE_NAME, level.as_str()).as_str())?
    }

    let threema_client = ThreemaClient::new(
        &cfg.threema.gateway_own_id,
        &cfg.threema.secret,
        &cfg.threema.private_key,
    )?;

    let homeserver_url = Url::parse(&cfg.matrix.homeserver_url)?;
    let matrix_client = Client::new(homeserver_url).await?;

    let app_state = web::Data::new(AppState {
        threema_client: threema_client.clone(),
        matrix_client: Mutex::new(matrix_client.clone()),
    });

    matrix_client
        .login(
            &cfg.matrix.user,
            &cfg.matrix.password,
            None,
            Some("command bot"),
        )
        .await?;

    matrix_client
        .sync_once(SyncSettings::default())
        .await
        .unwrap();
    matrix_client
        .register_event_handler_context(threema_client.clone())
        .register_event_handler(matrix_incoming_message_handler)
        .await;

    matrix_client
        .register_event_handler(on_stripped_state_member)
        .await;

    let settings = SyncSettings::default().token(matrix_client.sync_token().await.unwrap());

    let threema_server = tokio::spawn(
        HttpServer::new(move || {
            App::new().app_data(app_state.clone()).route(
                "/callback",
                web::post().to(threema_incoming_message_handler),
            )
        })
        .bind((
            cfg.threema.host.unwrap_or("localhost".to_owned()),
            cfg.threema.port.unwrap_or(443),
        ))?
        .run(),
    );

    let matrix_server = tokio::spawn(async move { matrix_client.sync(settings).await });

    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                matrix_server.abort();
                threema_server.abort();
                process::exit(1);
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
