use std::any::Any;
use std::array::TryFromSliceError;
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;
use crate::subtypes::Location;

#[derive(Error, Debug)]
pub enum KittyMCError {
    #[error("packet type hasn't been implemented")]
    NotImplemented(usize, usize), // packet_id packet_len
    #[error("failed to deserialize packet")]
    DeserializationError,
    #[error("failed to decode string in packet")]
    StringDecodeError(#[from] FromUtf8Error),
    #[error("not enough data: {0}<{1}")]
    NotEnoughData(usize, usize), // Actual, Required
    #[error("more data than was expected: {0}>{1}")]
    TooMuchData(usize, usize), // Actual, Required
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    #[error("{0}")]
    ByteConversionError(#[from] TryFromSliceError),
    #[error("{0}")]
    UuidConversionError(#[from] uuid::Error),
    #[error("OOMFIE happened :< : {0}")]
    OomfieError(&'static str),
    #[error("The bridge between the client and the server had an interruption")]
    ServerBridgeError,
    #[error("Still polling. Wait.")]
    Waiting,
    #[error("The client has disconnected")]
    Disconnected,
    #[error("The client version mismatched the server version")]
    VersionMissmatch,
    #[error("Data couldn't be decompressed properly")]
    DecompressionError, // No #[from] because DecompressError is stupip
    #[error("Couldn't deserialize type \"{0}\". Needed {1} bytes, got {2}")]
    NotEnoughBytesToDeserialize(&'static str, usize, usize),
    #[error("Couldn't deserialize variable type \"{0}\"")]
    VarDeserializationError(&'static str),
    #[error("The packet length was smaller than the header")]
    PacketLengthTooSmall,
    #[error("The packet length was invalid")]
    InvalidPacketLength,
    #[error("Zlib Decompression failed with error: {0}")]
    ZlibDecompressionError(miniz_oxide::inflate::DecompressError),
    #[error("The decompressed packet size was different than previously announced. Assuming corruption. {0} != {1}"
    )]
    InvalidDecompressedPacketLength(usize, usize), // Announced, Actual
    #[error("Thread exited unexpectedly: {0:?}")]
    ThreadError(Box<dyn Any + Send>),
    #[error("The requested client was not found")]
    ClientNotFound,
    #[error("The lock couldn't be locked")]
    LockPoisonError,
    #[error("The requested chunk position at {0} is invalid.")]
    InvalidChunk(Location),
    #[error("The requested block position at {0} is invalid.")]
    InvalidBlock(Location),
}
