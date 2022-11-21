extern crate serialport;

use std::error::Error as StdError;
use std::fmt;

use thiserror::Error;

// Differents file which should be linked
pub mod communicator;
pub mod crc8;
pub mod eep;
pub mod enocean;
pub mod frame;

/// Custom Result type = std::result::Result<T, ParseEspError>
type ParseEspResult<T> = std::result::Result<T, ParseEspError>;

/// Custom error type (eg. allow to see corresponding packet / byte index )
#[derive(Debug, Clone)]
pub struct ParseEspError {
    /// ErrorKind
    pub kind: ParseEspErrorKind,
    /// Associated message
    pub message: String,
    /// Index of the byte which caused the error
    pub byte_index: Option<i16>,
    /// Packet which caused this error
    pub packet: Vec<u8>,
}
/// Kind of error
#[derive(Debug, Clone, PartialEq)]
pub enum ParseEspErrorKind {
    NoSyncByte,
    CrcMismatch,
    IncompleteMessage,
    Unimplemented,
}

#[derive(Debug, Error)]
pub enum PacketReadError {
    #[error("IO Error")]            IOError(#[from] std::io::Error),
    #[error("End of Stream")]       EOF,
    #[error("Bad CRC for data")]    DataCRC(u8),
    #[error("Unknown packet type")] UnknownPacketType(u8),
}

impl fmt::Display for ParseEspError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.byte_index {
            // Error chould occur on a specific byte
            Some(bi) => write!(
                f,
                "{:?} error :{} in {:x?} at index {}",
                self.kind, self.message, self.packet, bi
            ),
            // Or on whole packet
            _ => write!(
                f,
                "{:?} error :{} in packet {:x?}",
                self.kind, self.message, self.packet
            ),
        }
    }
}
impl StdError for ParseEspError {
    fn description(&self) -> &str {
        &self.message
    }
}

/// Working with the type EnoceanMessage is more explicit than u8 vector.
type EnoceanMessage = Vec<u8>;
