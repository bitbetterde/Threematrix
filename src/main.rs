use actix_web::{App, HttpServer, web};
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::reqwest::Url;
use tokio::sync::Mutex;

use threematrix::{AppState, matrix_incoming_message_handler, threema_incoming_message_handler, ThreemaConfig};
use threematrix::threema::ThreemaClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Threematrix Server");

    let cfg = ThreemaConfig::new("./threema_gateway_cfg.toml");
    let threema_client = ThreemaClient::new(&cfg.gateway_own_id, &cfg.secret, &cfg.private_key);


    let homeserver_url = Url::parse("HOMESERVER").expect("Couldn't parse the homeserver URL");
    let user = "threematrix";
    let password = "PASSWORD";
    let matrix_client = Client::new(homeserver_url).await.unwrap();

    let app_state = web::Data::new(AppState { threema_client: threema_client.clone(), matrix_client: Mutex::new(matrix_client.clone()) });


    matrix_client.login(user, password, None, Some("command bot")).await.unwrap();

    // client.sync_once(SyncSettings::default()).await.unwrap();
    matrix_client
        .register_event_handler_context(threema_client.clone())
        .register_event_handler(matrix_incoming_message_handler).await;


    // let settings = SyncSettings::default().token(client.sync_token().await.unwrap());

    let (first, _) = tokio::join!(
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/callback", web::post().to(threema_incoming_message_handler))
    })
        .bind(("127.0.0.1", 8888))?
        .run(),
        matrix_client.sync(SyncSettings::default()));
    first.unwrap();
    Ok(())
}
