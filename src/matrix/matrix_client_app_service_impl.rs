use std::collections::btree_map::BTreeMap;
use crate::errors::{BindThreemaGroupToMatrixError, SendToMatrixRoomByThreemaGroupIdError};
use crate::matrix::util::{get_threematrix_room_state, set_threematrix_room_state, ThreematrixStateEventContent};
use crate::threema::util::convert_group_id_to_readable_string;
use async_trait::async_trait;
use log::{debug, warn};
use matrix_sdk::config::SyncSettings;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk_appservice::ruma::{assign, RoomId, TransactionId, UserId, OwnedUserId, Int};
use matrix_sdk_appservice::ruma::api::client::room::create_room::v3::Request as CreateRoomRequest;
use matrix_sdk_appservice::AppService;
use matrix_sdk_appservice::ruma::api::client::room::Visibility;
use matrix_sdk_appservice::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk_appservice::ruma::events::room::power_levels::RoomPowerLevelsEventContent;

use super::MatrixClient;

#[async_trait]
impl MatrixClient for AppService {
    async fn send_message_by_threema_group_id(
        &self,
        threema_group_id: &[u8],
        user_name: &str,
        user_id: &str,
        body: &str,
        html_body: &str,
    ) -> Result<(), SendToMatrixRoomByThreemaGroupIdError> {
        let content = RoomMessageEventContent::text_html(body, html_body);
        let mut room_found = false;

        self.register_virtual_user(format!("_threema_{}", user_id).as_str(), None).await;

        let admin_user = self
            .virtual_user(None)
            .await
            .map_err(|e| SendToMatrixRoomByThreemaGroupIdError::MatrixAppServiceError(e))?;


        let client = self
            .virtual_user(Some(format!("_threema_{}", user_id).as_str()))
            .await
            .map_err(|e| SendToMatrixRoomByThreemaGroupIdError::MatrixAppServiceError(e))?;

        client.account().set_display_name(Some("Bratwurst"));

        debug!("Matrix admin joined rooms: {:?}", admin_user.rooms());
        debug!("Matrix user joined rooms: {:?}", client.joined_rooms());

        for room in admin_user.joined_rooms() {
            match get_threematrix_room_state(&room).await {
                Ok(None) => debug!(
                    "Matrix: Room {:?} does not have proper room state",
                    &room
                        .display_name()
                        .await
                        .unwrap_or(matrix_sdk::DisplayName::Named("UNKNOWN".to_owned()))
                ),
                Ok(Some(state)) => {
                    debug!("Matrix: Room found");
                    client.join_room_by_id(room.room_id()).await
                        .map_err(|e| SendToMatrixRoomByThreemaGroupIdError::MatrixAppServiceHttpError(e))?;
                    let room = client.get_joined_room(room.room_id()).unwrap();

                    if let Ok(group_id) = convert_group_id_to_readable_string(&threema_group_id) {
                        if state.threematrix_threema_group_id == group_id {
                            let txn_id = TransactionId::new();
                            room.send(content.clone(), Some(&txn_id))
                                .await
                                .map_err(|e| {
                                    SendToMatrixRoomByThreemaGroupIdError::MatrixError(e)
                                })?;
                            room_found = true;
                        }
                    }
                }
                Err(e) => warn!("Matrix: Could not retrieve room state: {}", e),
            }
        }
        return if room_found {
            Ok(())
        } else {
            Err(SendToMatrixRoomByThreemaGroupIdError::NoRoomForGroupIdFoundError)
        };
    }

    async fn bind_threema_group_to_matrix_room(
        &self,
        threema_group_id: &[u8],
        matrix_room_id: &str,
    ) -> Result<(), BindThreemaGroupToMatrixError> {
        let room_id = <&RoomId>::try_from(matrix_room_id)
            .map_err(|e| BindThreemaGroupToMatrixError::InvalidMatrixRoomId(e))?;

        let client = self
            .virtual_user(None)
            .await
            .map_err(|e| BindThreemaGroupToMatrixError::MatrixAppServiceError(e))?;

        let mut power_levels = RoomPowerLevelsEventContent::new();
        let user_id = <OwnedUserId>::from(<&UserId>::try_from("@shorty1o1:fabcity.hamburg").unwrap());
        power_levels.users.insert(user_id.clone(), Int::new(100).unwrap());
        power_levels.users.insert(<OwnedUserId>::from(client.user_id().unwrap()), Int::new(100).unwrap());

        let invites = [user_id];

        let request = assign!(CreateRoomRequest::new(), {
            invite: &invites,
            visibility: Visibility::Public,
            name: Some("testThreema"),
            power_level_content_override: Some(Raw::new(&power_levels).unwrap())
        });
        let created_room = client.create_room(request).await.unwrap();

        client
            .join_room_by_id(&created_room.room_id)
            .await
            .map_err(|e| BindThreemaGroupToMatrixError::MatrixAppServiceHttpError(e))?;

        let room = client.get_joined_room(&created_room.room_id).unwrap();

        // let room = client.get_joined_room(room_id).unwrap();

        match convert_group_id_to_readable_string(&threema_group_id) {
            Ok(r) => {
                let content: ThreematrixStateEventContent = ThreematrixStateEventContent {
                    threematrix_threema_group_id: r,
                };

                if let Err(e) = set_threematrix_room_state(content, &room).await {
                    return Err(BindThreemaGroupToMatrixError::MatrixError(e));
                } else {
                    return Ok(());
                };
            }
            Err(e) => {
                return Err(BindThreemaGroupToMatrixError::InvalidGroupId(e));
            }
        }
    }
}
