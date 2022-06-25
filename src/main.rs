use std::sync::Mutex;

use actix_web::{App, HttpServer, web};

use threematrix::{AppState, incoming_message_handler, ThreemaConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Threematrix Server");

    let cfg = ThreemaConfig::new("./threema_gateway_cfg.toml");
    let app_state = web::Data::new(AppState
    {
        config: cfg,
        members: Mutex::new(Vec::new()),
        group_name: Mutex::new("".to_owned()),
        queued_messages: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/callback", web::post().to(incoming_message_handler))
    })
        .bind(("127.0.0.1", 8888))?
        .run()
        .await
}
