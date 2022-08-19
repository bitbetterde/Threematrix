use std::num::ParseIntError;
use std::string::FromUtf8Error;
use matrix_sdk::{Error, HttpError};
use matrix_sdk::ruma::IdParseError;
use thiserror::Error;
use threema_gateway::errors::{ApiError, CryptoError};

#[derive(Debug, Error)]
pub enum SendGroupMessageError {
    #[error("Members of group are unknown, because we haven't received any message in this group yet. Try sending a Threema message first.")]
    GroupNotInCache,
    #[error("{0}")]
    ApiError(ApiError),
}

#[derive(Debug, Error)]
pub enum ProcessIncomingMessageError {
    #[error("{0}")]
    CryptoError(CryptoError),
    #[error("{0}")]
    ApiError(ApiError),
    #[error("{0}")]
    Utf8ConvertError(FromUtf8Error),
    #[error("Unknown message type")]
    UnknownMessageTypeError,
}


#[derive(Debug, Error)]
pub enum StringifyGroupIdError {
    #[error("Group id is empty")]
    EmptyGroupId,
}

#[derive(Debug, Error)]
pub enum ParseGroupIdError {
    #[error("Group id should consist of 8 x u8 chars, separated by spaces")]
    InvalidGroupIdLength,
    #[error("Group id chars should be between 0 and 255 : {0}")]
    EncodingError(ParseIntError),
}

#[derive(Debug, Error)]
pub enum SendToMatrixRoomByThreemaGroupIdError {
    #[error("No Matrix room for group id found.")]
    NoRoomForGroupIdFoundError,
    #[error("{0}")]
    MatrixError(Error),
    #[error("{0}")]
    MatrixAppServiceError(matrix_sdk_appservice::Error),
    #[error("{0}")]
    MatrixAppServiceHttpError(HttpError),
}

#[derive(Debug, Error)]
pub enum BindThreemaGroupToMatrixError {
    #[error("{0}")]
    InvalidGroupId(StringifyGroupIdError),
    #[error("{0}")]
    MatrixError(Error),
    #[error("{0}")]
    MatrixAppServiceError(matrix_sdk_appservice::Error),
    #[error("{0}")]
    MatrixAppServiceHttpError(HttpError),
    #[error("{0}")]
    InvalidMatrixRoomId(IdParseError),
}
