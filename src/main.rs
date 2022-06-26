use actix_web::{App, HttpServer, web};

use threematrix::{AppState, incoming_message_handler, ThreemaConfig};
use threematrix::threema::ThreemaClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Threematrix Server");

    let cfg = ThreemaConfig::new("./threema_gateway_cfg.toml");
    let client = ThreemaClient::new(&cfg.gateway_own_id, &cfg.secret, &cfg.private_key);
    let app_state = web::Data::new(AppState { threema_client: client });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/callback", web::post().to(incoming_message_handler))
    })
        .bind(("127.0.0.1", 8888))?
        .run()
        .await
}
