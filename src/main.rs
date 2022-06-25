use actix_web::{web, App, HttpServer};
use threematrix::{incoming_message_handler, ThreemaConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Threematrix Server");

    let cfg = ThreemaConfig::new("./threema_gateway_cfg.toml");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cfg.clone()))
            .route("/callback", web::post().to(incoming_message_handler))
    })
    .bind(("127.0.0.1", 8888))?
    .run()
    .await
}
