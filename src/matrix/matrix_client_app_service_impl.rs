use crate::errors::{BindThreemaGroupToMatrixError, SendToMatrixRoomByThreemaGroupIdError};
use crate::matrix::util::{get_threematrix_room_state, set_threematrix_room_state, ThreematrixStateEventContent};
use crate::threema::util::convert_group_id_to_readable_string;
use async_trait::async_trait;
use log::{debug, warn};
use matrix_sdk_appservice::ruma::{RoomId, TransactionId};
use matrix_sdk_appservice::AppService;
use matrix_sdk_appservice::ruma::events::room::message::RoomMessageEventContent;

use super::MatrixClient;

#[async_trait]
impl MatrixClient for AppService {
    async fn send_message_by_threema_group_id(
        &self,
        threema_group_id: &[u8],
        user_name: &str,
        body: &str,
        html_body: &str,
    ) -> Result<(), SendToMatrixRoomByThreemaGroupIdError> {
        let content = RoomMessageEventContent::text_html(body, html_body);
        let mut room_found = false;

        // self.register_virtual_user("_threema_bratwurst", None).await;

        let admin_user = self
            .virtual_user(None)
            .await
            .map_err(|e| SendToMatrixRoomByThreemaGroupIdError::MatrixAppServiceError(e))?;

        // let client = self
        //     .virtual_user(Some("_threema_bratwurst"))
        //     .await
        //     .map_err(|e| SendToMatrixRoomByThreemaGroupIdError::MatrixAppServiceError(e))?;


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
                    // client.join_room_by_id(room.room_id()).await
                    //     .map_err(|e| SendToMatrixRoomByThreemaGroupIdError::MatrixAppServiceHttpError(e))?;
                    // let room = client.get_joined_room(room.room_id()).unwrap();

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

        client
            .join_room_by_id(room_id)
            .await
            .map_err(|e| BindThreemaGroupToMatrixError::MatrixAppServiceHttpError(e))?;

        let room = client.get_joined_room(room_id).unwrap();

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
