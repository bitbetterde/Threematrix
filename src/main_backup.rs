use actix_web::{App, HttpResponse, HttpServer, Responder, web, post};
use actix_web::web::Query;
use matrix_sdk::{
    self,
    room::Room,
    ruma::events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        SyncMessageEvent,
    },
    Client, SyncSettings,
};
use matrix_sdk::reqwest::Url;
use threema_gateway::{ApiBuilder, IncomingMessage, SecretKey};

const SECRET: &'static str = "RQjXIbz5RosCD4Xi";
const PRIVATE_KEY: &'static str = "b7609539a2029b2cbbc294bee5844155f422dd9852a472e7e7c9f10e16bac8a7";

const FROM: &'static str = "*BITBET1";
const TO_GROUP_ID: &'static str = "8MVC794X";

const MATRIX_HOMESERVER_URL: &'static str = "https://matrix.fabcity.hamburg";
const MATRIX_USERNAME: &'static str = "threematrix";
const MATRIX_PASSWORD: &'static str = "W.4E9oTa!sMH.Tcx";

#[post("/")]
async fn incoming_message(args: Query<IncomingMessage>) -> impl Responder {
    let our_id = &args.to;
    // let PRIVATE_KEY = HEXLOWER_PERMISSIVE
    //     .decode(args.get_str("<private-key>").as_bytes())
    //     .ok()
    //     .and_then(|bytes| SecretKey::from_slice(&bytes))
    //     .unwrap_or_else(|| {
    //         eprintln!("Invalid private key");
    //         std::process::exit(1);
    //     });
    // let request_body = &args.box_data;

    // Create E2eApi instance
    let api = ApiBuilder::new(our_id, SECRET)
        .with_private_key_bytes(PRIVATE_KEY.as_ref())
        .and_then(|builder| builder.into_e2e())
        .unwrap();


    let msg = args.into_inner();
    // api
    // .decode_incoming_message(request_body)
    // .unwrap_or_else(|e| {
    //     eprintln!("Could not decode incoming message: {}", e);
    //     std::process::exit(1);
    // });


    println!("Parsed and validated message from request:");
    println!("  From: {}", msg.from);
    println!("  To: {}", msg.to);
    println!("  Message ID: {}", msg.message_id);
    println!("  Timestamp: {}", msg.date);
    println!("  Sender nickname: {:?}", msg.nickname);

    // Fetch sender public key
    let pubkey = api.lookup_pubkey(&msg.from).await.unwrap_or_else(|e| {
        eprintln!("Could not fetch public key for {}: {}", &msg.from, e);
        std::process::exit(1);
    });

    // Decrypt
    let data = api
        .decrypt_incoming_message(&msg, &pubkey)
        .unwrap_or_else(|e| {
            println!("Could not decrypt box: {}", e);
            std::process::exit(1);
        });

    // Show result
    println!("Decrypted box: {:?}", data);

    HttpResponse::Ok().body("Hello world!")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Start Server");
    tokio::join!(HttpServer::new(|| {
        App::new()
            .service(incoming_message)
    })
        .bind(("127.0.0.1", 8080))?
        .run(),
    login(MATRIX_HOMESERVER_URL, MATRIX_USERNAME, MATRIX_PASSWORD));

    Ok(())
}

async fn on_room_message(event: SyncMessageEvent<MessageEventContent>, room: Room) {
    if let Room::Joined(room) = room {
        if let SyncMessageEvent {
            content:
            MessageEventContent {
                msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                ..
            },
            sender,
            ..
        } = event
        {
            let member = room.get_member(&sender).await.unwrap().unwrap();
            let name = member.display_name().unwrap_or_else(|| member.user_id().as_str());

            // Create E2eApi instance
            let api = ApiBuilder::new(FROM, SECRET)
                .with_private_key_str(PRIVATE_KEY)
                .and_then(|builder| builder.into_e2e())
                .unwrap();

            // Fetch public key
            // Note: In a real application, you should cache the public key
            let public_key = api.lookup_pubkey(TO_GROUP_ID).await.unwrap();

            // Encrypt
            let encrypted = api.encrypt_text_msg(format!("{}: {}", name, msg_body).as_str(), &public_key.into());

            // Send
            match api.send(TO_GROUP_ID, &encrypted, false).await {
                Ok(msg_id) => println!("Sent. Message id is {}.", msg_id),
                Err(e) => println!("Could not send message: {:?}", e),
            }
            println!("{}: {}", name, msg_body);
        }
    }
}

async fn login(
    homeserver_url: &str,
    username: &str,
    password: &str,
) -> Result<(), matrix_sdk::Error> {
    let homeserver_url = Url::parse(&homeserver_url).expect("Couldn't parse the homeserver URL");
    let client = Client::new(homeserver_url).unwrap();

    client.register_event_handler(on_room_message).await;

    client.login(username, password, None, Some("rust-sdk")).await?;
    client.sync(SyncSettings::new()).await;

    Ok(())
}