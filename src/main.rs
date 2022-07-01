use std::process;
use actix_web::{App, HttpServer, web};
use futures::stream::StreamExt;
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::reqwest::Url;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::sync::Mutex;

use threematrix::{
    AppState, matrix_incoming_message_handler, threema_incoming_message_handler, ThreematrixConfig,
};
use threematrix::threema::ThreemaClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Threematrix Server");
    let mut signals = Signals::new(&[
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ])?;


    let cfg = ThreematrixConfig::new("./threematrix_cfg.toml");
    let threema_client = ThreemaClient::new(
        &cfg.threema.gateway_own_id,
        &cfg.threema.secret,
        &cfg.threema.private_key,
    );

    let homeserver_url = Url::parse(&cfg.matrix.homeserver_url).expect("Couldn't parse the homeserver URL");
    let matrix_client = Client::new(homeserver_url).await.unwrap();

    let app_state = web::Data::new(AppState {
        threema_client: threema_client.clone(),
        matrix_client: Mutex::new(matrix_client.clone()),
    });

    matrix_client
        .login(&cfg.matrix.user, &cfg.matrix.password, None, Some("command bot"))
        .await
        .unwrap();

    // client.sync_once(SyncSettings::default()).await.unwrap();
    matrix_client
        .register_event_handler_context(threema_client.clone())
        .register_event_handler(matrix_incoming_message_handler)
        .await;

    // let settings = SyncSettings::default().token(client.sync_token().await.unwrap());

    let threema_server = tokio::spawn(HttpServer::new(move || {
        App::new().app_data(app_state.clone()).route(
            "/callback",
            web::post().to(threema_incoming_message_handler),
        )
    })
        .bind(("127.0.0.1", 8888))?
        .run());


    let matrix_server = tokio::spawn(async move {
        matrix_client.sync(SyncSettings::default()).await
    });

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
