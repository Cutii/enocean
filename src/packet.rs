//! ESP3 packet encoding and decoding

use std::str::Utf8Error;

use num_enum::TryFromPrimitive;
use thiserror::Error;

use crate::frame::{ESP3Frame, ESP3FrameRef};

pub type ResponseCode = crate::enocean::ReturnCode;

#[derive(Debug,Clone,Copy)]
pub struct Address([u8; 4]);

pub const BROADCAST: Address = Address([0xff,0xff,0xff,0xff]);

pub struct EEPProfileCode([u8; 3]);

#[derive(Debug,Error)]
pub enum ParseError {
    #[error("Unsupported packet type")] UnsupportedPacketType,
    #[error("Packet too short")]        PacketTooShort,
    #[error("UTF8 decoding Error")]     UTF8(#[from] Utf8Error),
    #[error("Invalid result code")]     InvalidResultCode(u8),
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SubtelNum {
    Send = 3, 
    Receive = 0,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Security {
    None = 0,
    Obsolete = 1,
    Decrypted = 2,
    Authenticated = 3,
    AuthAndDecrypted = 4,
}

#[derive(Debug,Clone,Copy)]
pub struct RadioErp1<'a> {
    pub choice: u8,
    pub user_data: &'a [u8],
    pub sender_id: Address,
    pub status: u8,
    pub subtel_num: Option<SubtelNum>,
    pub destination: Option<Address>,
    pub rssi: Option<u8>,
    pub security: Option<Security>
}

#[derive(Debug,Clone,Copy)]
// TODO parse details
pub enum Event<'a> {
    SAReclaimUnsuccessful,
    SAConfirmLearn       { data: &'a [u8; 17] }, 
    SALearnAck           { data: &'a [u8; 3]},
    COReady              { wakeup: u8, mode: Option<u8> },
    COEventSecureDevices { cause: u8, device: Address },
    CODutyCycleLimit     { cause: u8},
    COTXFailed           { cause: u8},
    COTXDone,
    COLrnModeDisabled,
}

#[derive(Debug,Clone)]
pub struct Response {
    pub code: ResponseCode,
    pub data: Vec<u8>,
}

#[derive(Debug,Clone,Copy)]
pub struct Version {
    pub main: u8,
    pub beta: u8,
    pub alpha: u8,
    pub build: u8,
}

#[derive(Debug,Clone)]
pub struct VersionResponse {
    pub app: Version,
    pub api: Version,
    pub chip_id: Address,
    pub chip_version: [u8; 4],
    pub description: String,
}

#[derive(Debug,Clone,Copy)]
pub enum CommonCommand<'a> {
    //Reset,
    ReadVersion,
    //ReadSystemLog,

    Unknown { code: u8, data: &'a [u8], optional: &'a [u8] }
}

#[derive(Debug,Clone)]
pub enum Packet<'a> {
    //RadioErp1(RadioErp1<'a>),
    Response(Response),
    //Event(Event<'a>),
    CommonCommand(CommonCommand<'a>),
    //SmartAck,
    //RemoteMan,
    //RadioMessage,
    //RadioErp2,
    //CommandAccepted,
    //RadioLRWPAN,
    //Command24GHz,

    Unknown { packet_type: u8, data: &'a [u8], optional: &'a [u8] }
    //RadioSubTel(RadioSubTel),
}

impl VersionResponse {
    pub fn encode(&self) -> Response {
        todo!();
    }

    pub fn decode(response: &Response) -> Result<Self, ParseError> {
        let d = &response.data;
        if d.len() != 32 {
            return Err(ParseError::PacketTooShort)
        }

        Ok(Self {
            app: Version { main: d[0], beta: d[1], alpha: d[2], build: d[3] },
            api: Version { main: d[4], beta: d[5], alpha: d[6], build: d[7] },
            chip_id: Address(d[8..12].try_into().unwrap()),
            chip_version: d[12..16].try_into().unwrap(),
            description: std::str::from_utf8(&d[16..32])?.to_owned(),
        })

    }
}

impl Response {

    pub fn encode(&self) -> ESP3Frame {
        todo!()
    }

    pub fn decode(frame: ESP3FrameRef) -> Result<Self, ParseError> {
        let code = ResponseCode::try_from_primitive(frame.data[0])
            .map_err(|_| ParseError::InvalidResultCode(frame.data[0]))?;
        let data = frame.data[1..].into();
        Ok( Self { code, data })
    }

}

impl<'a> CommonCommand<'a> {

    fn assemble(code: u8, data: &[u8], optional: &[u8]) -> ESP3Frame {
        let packet_type = 0x05;
        let mut frame_data = vec![code];
        frame_data.extend_from_slice(data);
        ESP3Frame::assemble(packet_type, &frame_data, optional)
    }

    fn encode(&self) -> ESP3Frame {
        match self {
            &Self::Unknown { code, data, optional } => CommonCommand::assemble(code, data, optional),
            &Self::ReadVersion => CommonCommand::assemble(0x03, &[], &[]),
        }
    }
}

impl<'a> Packet<'a> {
    pub fn encode(&self) -> ESP3Frame {

        use Packet::*;
        match &self {
            &CommonCommand(cmd) => cmd.encode(),
            &Response(resp) => resp.encode(),
            &Unknown { packet_type, data, optional } => ESP3Frame::assemble(*packet_type, data, optional),
        }       
    }

    pub fn decode(frame: ESP3FrameRef<'a>) -> Result<Self, ParseError> {
        match frame.packet_type {
            0x02 => Ok(Self::Response(Response::decode(frame)?)),
            _    => Err(ParseError::UnsupportedPacketType),
        }
    }

}

