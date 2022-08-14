use std::error::Error;
use std::process;

use actix_web::{web, App, HttpServer};
use flexi_logger::Logger;
use futures::stream::StreamExt;
use log::{debug, info};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::reqwest::Url;
use matrix_sdk::Client;
use matrix_sdk_appservice::matrix_sdk::event_handler::Ctx;
use matrix_sdk_appservice::matrix_sdk::room::Room;
use matrix_sdk_appservice::ruma::events::room::member::OriginalSyncRoomMemberEvent;
use matrix_sdk_appservice::{AppService, AppServiceRegistration};
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::sync::Mutex;

use threematrix::incoming_message_handler::matrix_app_service::{handle_room_member, matrix_app_service_incoming_message_handler};
use threematrix::incoming_message_handler::threema::threema_incoming_message_handler;
use threematrix::threema::ThreemaClient;
use threematrix::{AppState, LoggerConfig, ThreematrixConfig};
use threematrix::incoming_message_handler::matrix_user::matrix_user_incoming_message_handler;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut logger = Logger::try_with_str(format!("{}=info", CRATE_NAME))?.start()?;

    let cfg = ThreematrixConfig::new("./threematrix_cfg.toml");
    info!(
        "Starting Threematrix Server v{}. Waiting for Threema callback on {}:{}",
        VERSION,
        cfg.threema.host.clone().unwrap(),
        cfg.threema.port.clone().unwrap()
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

    let app_state;
    let matrix_server;

    let matrix_mode = cfg.matrix.mode.unwrap();

    if matrix_mode == "user" {
        let matrix_client = Client::new(homeserver_url.clone()).await?;
        matrix_client
            .login(
                &cfg.matrix.user,
                &cfg.matrix.password,
                None,
                Some("Threematrix Bot"),
            )
            .await?;

        debug!("Matrix: Successfully logged in as client");

        matrix_client
            .sync_once(SyncSettings::default())
            .await
            .unwrap();

        debug!("Matrix: Initial sync successful");

        matrix_client
            .register_event_handler_context(threema_client.clone())
            .register_event_handler(matrix_user_incoming_message_handler)
            .await;
        let settings = SyncSettings::default().token(matrix_client.sync_token().await.unwrap());

        app_state = web::Data::new(AppState {
            threema_client: threema_client.clone(),
            matrix_client: Mutex::new(Box::new(matrix_client.clone())),
        });

        matrix_server = tokio::spawn(async move { matrix_client.sync(settings).await });
    } else {
        debug!("Matrix: Starting app service");
        let registration = AppServiceRegistration::try_from_yaml_file("./registration.yaml")?;
        let appservice = AppService::new(
            cfg.matrix.homeserver_url.as_str(),
            "fabcity.hamburg",
            registration,
        )
            .await?;

        let virtual_user = appservice.virtual_user(None).await?;

        virtual_user.add_event_handler_context(appservice.clone());
        virtual_user.add_event_handler_context(threema_client.clone());
        debug!("Matrix: Init Virtual User");
        virtual_user
            .add_event_handler(
                move |event: OriginalSyncRoomMemberEvent,
                      room: Room,
                      Ctx(appservice): Ctx<AppService>| {
                    debug!("Matrix: OriginalSyncRoomMemberEvent received");
                    handle_room_member(appservice, room, event)
                },
            )
            .await;

        virtual_user
            .add_event_handler(matrix_app_service_incoming_message_handler)
            .await;

        debug!("Matrix: Virtual User Event Handler Added");

        app_state = web::Data::new(AppState {
            threema_client: threema_client.clone(),
            matrix_client: Mutex::new(Box::new(appservice.clone())),
        });

        matrix_server = tokio::spawn(async move {
            let (host, port) = appservice.registration().get_host_and_port().unwrap();
            debug!("Matrix: Start Server on {}:{}", host, port);
            appservice.run(host, port).await.unwrap();
        });
    }

    let threema_server = tokio::spawn(
        HttpServer::new(move || {
            App::new().app_data(app_state.clone()).route(
                "/callback",
                web::post().to(threema_incoming_message_handler),
            )
        })
            .bind((cfg.threema.host.unwrap(), cfg.threema.port.unwrap()))?
            .run(),
    );

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
