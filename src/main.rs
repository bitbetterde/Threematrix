use std::sync::Mutex;

use actix_web::{web, App, HttpServer};

use threematrix::{incoming_message_handler, AppState, ThreemaConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Threematrix Server");

    let cfg = ThreemaConfig::new("./threema_gateway_cfg.toml");
    let app_state = web::Data::new(AppState { config: cfg });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/callback", web::post().to(incoming_message_handler))
    })
    .bind(("127.0.0.1", 8888))?
    .run()
    .await
}
