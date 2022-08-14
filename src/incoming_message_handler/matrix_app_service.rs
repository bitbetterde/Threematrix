use log::{debug, error, trace, warn};
use matrix_sdk::event_handler::Ctx;
// use matrix_sdk::room::{Joined, Room};
use matrix_sdk::ruma::events::room::message::{MessageType, TextMessageEventContent};
use matrix_sdk::ruma::TransactionId;
use matrix_sdk_appservice::matrix_sdk::room::{Joined, Room};
use matrix_sdk_appservice::matrix_sdk::HttpError;
use matrix_sdk_appservice::ruma::api::client::{error::ErrorKind, uiaa::UiaaResponse};
use matrix_sdk_appservice::ruma::api::error::{FromHttpResponseError, ServerError};
use matrix_sdk_appservice::ruma::events::room::member::{
    MembershipState, OriginalSyncRoomMemberEvent,
};
use matrix_sdk_appservice::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk_appservice::ruma::events::OriginalSyncMessageLikeEvent;
use matrix_sdk_appservice::ruma::UserId;
use matrix_sdk_appservice::AppService;

use crate::matrix::util::get_threematrix_room_state;
use crate::threema::util::convert_group_id_from_readable_string;
use crate::threema::ThreemaClient;

pub async fn matrix_incoming_message_handler(
    event: OriginalSyncMessageLikeEvent<RoomMessageEventContent>,
    room: Room,
    threema_client: Ctx<ThreemaClient>,
    // matrix_client: Client,
) -> () {
    debug!("Matrix: OriginalSyncMessageLikeEvent received");
    match room {
        Room::Joined(room) => {
            if let OriginalSyncMessageLikeEvent {
                content:
                    RoomMessageEventContent {
                        msgtype: MessageType::Text(TextMessageEventContent { body: msg_body, .. }),
                        ..
                    },
                sender,
                ..
            } = event
            {
                debug!("Matrix: Incoming message: {}", msg_body);

                let sender_member = room.get_member(&sender).await;
                match sender_member {
                    Ok(Some(sender_member)) => {
                        let sender_name = sender_member
                            .display_name()
                            .unwrap_or_else(|| sender_member.user_id().as_str());

                        // Filter out messages coming from our own bridge user
                        // if sender != matrix_client.user_id().unwrap() {
                        match get_threematrix_room_state(&room).await {
                            Ok(None) => {
                                let err_txt = format!("Room {} does not have proper room state. Have you bound the room to a Threema group?",
                                                      &room.display_name().await.unwrap_or(matrix_sdk::DisplayName::Named("UNKNOWN".to_owned())));
                                send_error_message_to_matrix_room(&room, err_txt, false).await;
                            }
                            Ok(Some(threematrix_state)) => {
                                let group_id = convert_group_id_from_readable_string(
                                    threematrix_state.threematrix_threema_group_id.as_str(),
                                );

                                if let Ok(group_id) = group_id {
                                    if let Err(e) = threema_client
                                        .send_group_msg_by_group_id(
                                            format!("*{}*: {}", sender_name, msg_body).as_str(),
                                            group_id.as_slice(),
                                        )
                                        .await
                                    {
                                        let err_txt = format!(
                                            "Couldn't send message to Threema group: {}",
                                            e
                                        );
                                        send_error_message_to_matrix_room(&room, err_txt, true)
                                            .await;
                                    }
                                }
                            }
                            Err(e) => {
                                let err_txt = format!("Could not retrieve room state: {}", e);
                                send_error_message_to_matrix_room(&room, err_txt, true).await;
                            }
                        }
                        // }
                    }
                    _ => {
                        error!("Matrix: Could not resolve room member!");
                    }
                }
            }
        }
        _ => {
            // If bot not member of room, ignore incoming message
        }
    }
}

async fn send_error_message_to_matrix_room(room: &Joined, err_txt: String, log_level_err: bool) {
    if log_level_err {
        error!("Matrix: {}", err_txt);
    } else {
        warn!("Matrix: {}", err_txt);
    }

    let content = RoomMessageEventContent::text_plain(err_txt.clone());
    let txn_id = TransactionId::new();

    if let Err(e) = room.send(content, Some(&txn_id)).await {
        error!(
            "Matrix: Could not send error message: \"{}\". {}",
            err_txt, e
        )
    }
}

pub async fn handle_room_member(
    appservice: AppService,
    room: Room,
    event: OriginalSyncRoomMemberEvent,
) -> Result<(), matrix_sdk_appservice::Error> {
    if !appservice.user_id_is_in_namespace(&event.state_key) {
        trace!("not an appservice user: {}", event.state_key);
    } else if let MembershipState::Invite = event.content.membership {
        let user_id = UserId::parse(event.state_key.as_str())?;
        if let Err(error) = appservice
            .register_virtual_user(user_id.localpart(), None)
            .await
        {
            error_if_user_not_in_use(error)?;
        }

        let client = appservice.virtual_user(Some(user_id.localpart())).await?;
        client.join_room_by_id(room.room_id()).await?;
    }

    Ok(())
}

pub fn error_if_user_not_in_use(
    error: matrix_sdk_appservice::Error,
) -> Result<(), matrix_sdk_appservice::Error> {
    match error {
        // If user is already in use that's OK.
        matrix_sdk_appservice::Error::Matrix(matrix_sdk::Error::Http(HttpError::UiaaError(
            FromHttpResponseError::Server(ServerError::Known(UiaaResponse::MatrixError(error))),
        ))) if matches!(error.kind, ErrorKind::UserInUse) => Ok(()),
        // In all other cases return with an error.
        error => Err(error),
    }
}
