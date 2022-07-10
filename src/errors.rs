use std::num::ParseIntError;
use std::string::FromUtf8Error;
use thiserror::Error;
use threema_gateway::errors::{ApiError, CryptoError};

#[derive(Debug, Error)]
pub enum SendGroupMessageError {
    #[error("Members of group are unknown, because we haven't received any message in this group yet.")]
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
    #[error("Unknown Message Type")]
    UnknownMessageTypeError,
}


#[derive(Debug, Error)]
pub enum StringifyGroupIdError {
    #[error("Group Id is empty")]
    EmptyGroupId,
}

#[derive(Debug, Error)]
pub enum ParseGroupIdError {
    #[error("Group Id should consist of 8 x u8 chars, separated by spaces")]
    InvalidGroupIdLength,
    #[error("Group Id chars should be between 0 and 255 : {0}")]
    EncodingError(ParseIntError),
}
