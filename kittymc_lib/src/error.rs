use std::array::TryFromSliceError;
use std::io;
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
    #[error("not enough data: {0}<{1}")]
    NotEnoughData(usize,usize),
    #[error("more data than was expected: {0}>{1}")]
    TooMuchData(usize,usize),
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    #[error("{0}")]
    ByteConversionError(#[from] TryFromSliceError),
}
