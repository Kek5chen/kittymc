use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KittyMCError {
    #[error("packet type hasn't been implemented")]
    NotImplemented,
    #[error("failed to decode packet")]
    DecodingError,
    #[error("failed to decode string in packet")]
    StringDecodeError(#[from] FromUtf8Error),
    #[error("not enough data collected for packet")]
    NotEnoughData,
}
