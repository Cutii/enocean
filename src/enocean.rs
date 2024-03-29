//!  # Enocean implementation for the Rust Programming Language
//!
//! Enocean : ([official website](https://www.enocean.com/en/)) is a Radio protocol for Smart Home / Buildings devices.
//!
//! This lib is a rust implementation of Enocean Serial Protocol, which you can find here: [ESP3](https://www.enocean.com/esp)
//! You can use this library with any compatible EnOcean Radio Gateway (eg. [USB300 gateway](https://www.enocean.com/en/enocean-modules/details/usb-300-oem/)).
//!
//!
//! :warning: **This lib is still under construction** :warning:
//!
//! ## Feature Overview
//! Enocean Radio protocol for Smart Homes rust implementation ([official website](https://www.enocean.com/en/))
//!
//!  EnOcean is a Radio protocol for SmartHome devices. More informations about EnOcean : [Official website](https://www.enocean.com/en/)
//! This lib allow you to play with Enocean Serial Protocol, which is defined here: [ESP3](https://www.enocean.com/esp)
//! You can use this library with any compatible EnOcean Radio Gateway eg. [USB300 gateway](https://www.enocean.com/en/enocean-modules/details/usb-300-oem/).
//!
//! For now this lib allow you to create an ESP struct from an incomming bytes vector.
//!
//! **Supported packet type** :
//!  - [x]   Radio ERP1 : 0x01  
//!  - [x]   Response : 0x02  
//!  - [ ]   radio_sub_tel : 0x03  
//!  - [ ]   event : 0x04  
//!  - [ ]   common_command : 0x05  
//!  - [ ]   smart_ack_command : 0x06  
//!  - [ ]   remote_man_command : 0x07  
//!  - [ ]   radio_message : 0x09  
//!  - [ ]   radio_advanced : 0x0a  
//!
//! ## License
//! [license]: #license
//!
//! This library is primarily distributed under the terms of both the MIT license
//! and the Apache License (Version 2.0).
//!
//! See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
//!

use num_enum::{TryFromPrimitive, IntoPrimitive};

use crate::*;

/// Simply clone the given u8 vector in an EnoceaMessage type variable
pub fn get_raw_message(em: Vec<u8>) -> EnoceanMessage {
    em.clone()
}
///  Main structure that represent an EnOcean Serial Packet according to ESP3 :  
///  
/// | Size (Byte) |   1    |       2          |        1      |      1    |      1    | u16 DataLen + u8 OptionLen |      1      |  
/// |-------------|--------|------------------|---------------|-----------|-----------|----------------------------|-------------|  
/// | Content     | 0x55   | u16DataLen       | u8OptionLen   | u8Type    | CRC8H     | DATA + OPT DATA *          |     CRC8D   |  
///  
/// Optionnal Data and Data layout/content depend on the Packet Type Identifier (4th byte)  
///  
/// ## Possible packet types :  
///  
/// #### Packet_type 0x01 - ERP1 : enocean serial packet representing a radio message.  
/// The Enocean Radio Packet v1 (ERP1) define more informations about the layout of the ESP3 packet :  
///  
///  - Data :  
///  
/// | Size (Byte) |        1      |         1              |       4        |   1      |  
/// |-------------|---------------|------------------------|----------------|----------|  
/// | Content     | Rorg (0xD5)   | Data payload as EEP*   | Sender ID      | Status   |  
///  
/// Data Payload length and content depend on the Enocean Equipment Profile of the device.  
///  
///  - Optionnal data :  
///  
/// | Size (Byte) |      1      |   4              |    1     |          1       |  
/// |-------------|-------------|------------------|----------|------------------|  
/// | Content     | Subtel nb   | Destination ID   | dBm      | Security level   |  
///  
/// #### Packet_type 0x02 - Response :   
/// ToDo : parse Response from TCM, eg. after trying to send a message as mentionned in the ESP3 :  
/// > When receiving a telegram, no RESPONSE has to be sent.  
/// > When sending a telegram, a RESPOND has to be expected. In this case, the following RESPONSE message gives the return codes:  
/// > OO RET_OK  
/// > O2 RET_NOT_SUPPORTED  
/// > 03 RET_WRONG_PARAM  
/// > 05 RET_LOCK_SET  
///
/// #### Other packet types :   
/// May be implemented later :     
/// [ ] radio_sub_tel : 0x03      
/// [ ] event : 0x04    
/// [ ] common_command : 0x05    
/// [ ] smart_ack_command : 0x06    
/// [ ] remote_man_command : 0x07    
/// [ ] radio_message : 0x09    
/// [ ] radio_advanced : 0x0a    

/// ESP3 struct is the representation of an Enocean Serial Packet.  
/// See [ESP3 protocol](https://www.enocean.com/esp) for more informations  
#[derive(Debug, PartialEq, Clone)]
pub struct ESP3 {
    // ESP3 packet structure, data and opt-data depend on packet_type
    data_length: u16,
    optional_data_length: u8,
    packet_type: PacketType,
    pub data: DataType,
    opt_data: Option<OptDataType>,
    crc_header: u8,
    crc_data: u8,
}
/// Util function to display packet information. Maybe we have to impl display for ESP3 instead ?
impl fmt::Display for ESP3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.data {
            DataType::Erp1Data {
                rorg,
                sender_id,
                status,
                payload,
            } => {
                write!(f,"{:X?} radio message from: {:X?} with Status {:X?} and Payload: {:X?}. \n Parsed Payload : \n {:#X?}"
                , rorg, sender_id, status, payload, enocean::eep::parse_erp1_payload(self).unwrap_or_default())
            }
            DataType::ResponseData {
                return_code,
                response_payload,
            } => {
                match response_payload {
                    Some(payload) => {
                        write!(f,"Response from TCM300 with RC : {:X?}. Optionnal payload : {:X?}", *return_code as u8, payload )
                    }
                    None => {
                            write!(f,"Response from TCM300 with RC : {:X?} and no payload", *return_code as u8 )
                    }
                }
            }
            DataType::RawData { raw_data } => {
                write!(f,"Unknow message: {:X?}", raw_data)
            }
        }
    }
}
/// Function to transform an ESP3 packet to an u8 vector.
impl From<&ESP3> for Vec<u8> {
    fn from(esp3 : &ESP3) -> Vec<u8> {
    let mut esp3_vector: EnoceanMessage = vec![0x55];
    esp3_vector.push((esp3.data_length >> 8) as u8);
    esp3_vector.push((esp3.data_length) as u8);
    esp3_vector.push(esp3.optional_data_length);
    esp3_vector.push(esp3.packet_type as u8);
    esp3_vector.push(esp3.crc_header);

    match &esp3.data {
        DataType::Erp1Data {
            rorg,
            sender_id,
            status,
            payload,
        } => {
            esp3_vector.push(*rorg as u8);
            esp3_vector.extend_from_slice(&payload);
            esp3_vector.extend_from_slice(sender_id);
            esp3_vector.push(*status);
        }
        DataType::ResponseData {
            return_code,
            response_payload,
        } => {
            esp3_vector.push(*return_code as u8);
            match response_payload {
                Some(ref payload) => esp3_vector.extend_from_slice(payload),
                None => {}
            }
        }
        DataType::RawData { raw_data } => {
            esp3_vector.extend_from_slice(&raw_data);
        }
    };
    match &esp3.opt_data {
        Some(OptDataType::Erp1OptData {
            subtel_num,
            destination_id,
            rssi,
            security_lvl,
        }) => {
            esp3_vector.push(*subtel_num);
            esp3_vector.extend_from_slice(destination_id);
            esp3_vector.push(*rssi);
            esp3_vector.push(*security_lvl);
        }
        Some(OptDataType::RawData { raw_data }) => {
            esp3_vector.extend_from_slice(&raw_data);
        }
        None => {}
    };
    esp3_vector.push(esp3.crc_data);
    esp3_vector
    }
}

/// Depending on packet_type, data and opt_data part of an ESP3 is implemented differently
#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    RawData {
        raw_data: Vec<u8>,
    },
    Erp1Data {
        rorg: Rorg,
        sender_id: [u8; 4],
        status: u8,
        payload: Vec<u8>,
    },
    ResponseData {
        return_code: ReturnCode,
        response_payload: Option<Vec<u8>>,
    },
}
/// Depending on packet_type, data and opt_data part of an ESP3 is implemented differently
#[derive(Debug, PartialEq, Clone)]
pub enum OptDataType {
    RawData {
        raw_data: Vec<u8>,
    },
    Erp1OptData {
        subtel_num: u8,
        destination_id: [u8; 4],
        rssi: u8,
        security_lvl: u8,
    },
}

/// Simple implementation of EnOcean packet type for ESP3 packet
/// Supported packet type for now : Radio_ERP1, Response
#[derive(PartialEq, Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum PacketType {
    RadioErp1 = 0x01,
    Response = 0x02,
    Undefined = 0xFF,
    // Unimplemented at the moment :
    RadioSubTel = 0x03,
    Event = 0x04,
    CommonCommand = 0x05,
    SmartAckCommand = 0x06,
    RemoteManCommand = 0x07,
    RadioMessage = 0x09,
    RadioErp2 = 0x0A,
    Radio802_15_4 = 0x10,
    Command2_4 = 0x11,
}

/// Given an packet type u8 value, return the corresponding PacketType
fn get_packet_type(em: &[u8]) -> ParseEspResult<PacketType> {
    PacketType::try_from_primitive(em[4])
        .map_err(|_| {
            ParseEspError {
                message: String::from("Invalid or unimplemented yet packet type"),
                byte_index: Some(4),
                packet: em.to_vec(),
                kind: ParseEspErrorKind::Unimplemented,
            }
        })
}

/// Simple implementation of possible Radio Organization for a Radio ERP1 packet (from EnOcean ESP3)
#[derive(PartialEq, Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Rorg {
    Undefined = 0xFF,
    Rps = 0xF6,
    Bs1 = 0xD5,
    Bs4 = 0xA5,
    Vld = 0xD2,
    Msc = 0xD1,
    Adt = 0xA6,
    Ute = 0xD4,
    SmLrnReq = 0xC6,
    SmLrnAns = 0xC7,
    SmRec = 0xA7,
    SysEx = 0xC5,
    Sec = 0x30,
    SecEncaps = 0x31,
}
/// Simple implementation of possible Return codes for a response packet (from EnOcean ESP3)
#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ReturnCode {
    Ok = 0x00,
    Error = 0x01,
    NotSupported = 0x02,
    WrongParam = 0x03,
    OperationDenied = 0x04,
    LockSet = 0x05,
    BufferTooSmall = 0x06,
    NoFreeBuffer = 0x07,
    Undefined = 0xff,
}

fn get_return_code(rc_byte: u8) -> ReturnCode {
    ReturnCode::try_from_primitive(rc_byte).unwrap_or(ReturnCode::Undefined)
}

/// Given an u8 byte containing Rorg indicator, return the corresponding Rorg variant
fn get_radio_organization(rorg_byte: u8) -> Rorg {
    Rorg::try_from_primitive(rorg_byte).unwrap_or(Rorg::Undefined)
}

pub use crc8::compute_crc8;

/// Main function which convert an u8 vector of incoming byte into an ESP3 variable :
///
/// | Size (Byte) |   1    |       2          |        1      |      1    |      1    | u16 DataLen + u8 OptionLen |      1      |
/// |-------------|--------|------------------|---------------|-----------|-----------|----------------------------|-------------|
/// | Content     | 0x55   | u16DataLen       | u8OptionLen   | u8Type    | CRC8H     | DATAS                      |     CRC8D   |
///
/// /// Optionnal data :   
///
/// | Size (Byte) |      1      |   4              |    1     |          1       |    
/// |-------------|-------------|------------------|----------|------------------|    
/// | Content     | Subtel nb   | Destination ID   | dBm      | Security level   |     
///
/// Data (BS1 example) :  
///
/// | Size (Byte) |        1      |         1              |       4        |   1      |      
/// |-------------|---------------|------------------------|----------------|----------|   
/// | Content     | Rorg (0xD5)   | Data payload as EEP*   | Sender ID      | Status   |   


pub fn esp3_of_enocean_message(em: &[u8]) -> ParseEspResult<ESP3> {
    // Make some verifications about the received message
    if em[0] != 0x55 {
        // EnOcean message must start by 0x55
        return Err(ParseEspError {
            message: String::from("Sync Byte Error"),
            byte_index: Some(0),
            packet: em.into(),
            kind: ParseEspErrorKind::NoSyncByte,
        });
    } else if em.len() <= 7 {
        // Minimal EnOcean message size = 7 bytes
        return Err(ParseEspError {
            message: String::from("Invalid input message"),
            byte_index: None,
            packet: em.into(),
            kind: ParseEspErrorKind::IncompleteMessage,
        });
    }
    let crc_header = em[5];
    if compute_crc8(&em[1..5].to_vec()) != em[5] {
        // EnOcean message header CRC can be checked without complex parsing
        return Err(ParseEspError {
            message: String::from("CRC Error"),
            byte_index: Some(5),
            packet: em.into(),
            kind: ParseEspErrorKind::CrcMismatch,
        });
    }

    // As header seems OK, we can parse data and opt_data length fields :
    let data_length: u16 = (em[1] as u16) << 8 | em[2] as u16;
    let optional_data_length: u8 = em[3];

    // And so we can check header and data length :
    if em.len() < (data_length as usize + optional_data_length as usize + 7) {
        return Err(ParseEspError {
            message: String::from("Packet length error"),
            byte_index: None,
            packet: em.into(),
            kind: ParseEspErrorKind::IncompleteMessage,
        });
    }
    let crc_data =
        compute_crc8(&em[6..6 + data_length as usize + optional_data_length as usize].to_vec());
    // And DATA CRC :
    if crc_data != em[6 + data_length as usize + optional_data_length as usize] {
        return Err(ParseEspError {
            message: String::from("CRC Data Error"),
            byte_index: Some(em[6 + data_length as usize + optional_data_length as usize] as i16),
            packet: em.into(),
            kind: ParseEspErrorKind::CrcMismatch,
        });
    }

    // If Message seems valid, we can then parse packet type
    let mut packet_type = PacketType::Undefined;
    let data: DataType;
    let opt_data: Option<OptDataType>;

    // Depending on packet_type, we can parse more informations about the message
    match get_packet_type(&em) {
        Ok(pt) => {
            match pt {
                PacketType::RadioErp1 => {
                    // See ERP1 definition in Enocean Serial Protocol
                    packet_type = PacketType::RadioErp1;
                    let mut sender_id: [u8; 4] = Default::default();
                    sender_id
                        .copy_from_slice(&em[1 + data_length as usize..5 + data_length as usize]);
                    // Data of erp1 packet contains rorg, data payload, sender_id and status
                    data = DataType::Erp1Data {
                        rorg: get_radio_organization(em[6]),
                        sender_id,
                        status: em[5 + data_length as usize],
                        payload: em[7..1 + data_length as usize].to_vec(), //7 + data_length - 6
                    };
                    let mut destination_id: [u8; 4] = Default::default();
                    destination_id
                        .copy_from_slice(&em[7 + data_length as usize..11 + data_length as usize]);

                    opt_data = Some(OptDataType::Erp1OptData {
                        subtel_num: em[6 + data_length as usize],
                        destination_id,
                        rssi: em[11 + data_length as usize],
                        security_lvl: em[12 + data_length as usize],
                    })
                }
                PacketType::Response => {
                    let mut response_payload: Option<Vec<u8>> = None;
                    if data_length > 1 {
                        response_payload = Some(em[7..data_length as usize].to_vec());
                    }
                    data = DataType::ResponseData {
                        return_code: get_return_code(em[6]),
                        response_payload,
                    };
                    opt_data = None;
                }
                _ => {
                    data = DataType::RawData {
                        raw_data: em[6..6 + data_length as usize].to_vec(),
                    };
                    opt_data = Some(OptDataType::RawData {
                        raw_data: em[6 + data_length as usize
                            ..6 + data_length as usize + optional_data_length as usize]
                            .to_vec(),
                    })
                }
            }
        }
        Err(_e) => {
            return Err(ParseEspError {
                message: String::from("Packet type error / not implemented yet"),
                byte_index: Some(4),
                packet: em.into(),
                kind: ParseEspErrorKind::Unimplemented,
            });
        }
    }

    // Return an Ok(ESP3)
    Ok(ESP3 {
        data_length,
        optional_data_length,
        packet_type,
        data,
        opt_data,
        crc_header,
        crc_data,
    })
}

/// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    // Enocean Serial Protocol 3 : ESP3 typical fields
    // -------------------------------------------------------------------
    #[test]
    fn given_valid_a50401_enocean_message_then_return_esp_with_valid_header() {
        // received_message is a valid message from a temperature / Humidity sensor (EEP A5-04-01)
        let received_message = vec![
            85, 0, 10, 7, 1, 235, 165, 16, 8, 70, 128, 5, 17, 114, 247, 0, 1, 255, 255, 255, 255,
            65, 0, 235,
        ];
        let data_length: u16 = 10;
        let optionnal_length: u8 = 7;
        let packet_type = PacketType::RadioErp1;
        let result = esp3_of_enocean_message(&received_message).unwrap();
        assert_eq!(data_length, result.data_length);
        assert_eq!(optionnal_length, result.optional_data_length);
        assert_eq!(packet_type, result.packet_type);
    }
    #[test]
    fn given_valid_f60201_enocean_message_then_return_esp_with_valid_header() {
        // received_message is a valid message from a necklace pushbutton (EEP -00-01)
        let received_message = vec![
            85, 0, 7, 7, 1, 122, 246, 0, 254, 245, 143, 212, 32, 2, 255, 255, 255, 255, 48, 0, 39,
        ];
        let data_length: u16 = 7;
        let optionnal_length: u8 = 7;
        let packet_type = PacketType::RadioErp1;
        let result = esp3_of_enocean_message(&received_message).unwrap();
        assert_eq!(data_length, result.data_length);
        assert_eq!(optionnal_length, result.optional_data_length);
        assert_eq!(packet_type, result.packet_type);
    }
    #[test]
    fn given_valid_a50401_message_with_valid_header_then_return_esp_with_valid_crc_header() {
        // received_message is a valid message from a necklace pushbutton (EEP -00-01)
        let received_message = vec![
            85, 0, 7, 7, 1, 122, 246, 0, 254, 245, 143, 212, 32, 2, 255, 255, 255, 255, 48, 0, 39,
        ];
        let crc_header: u8 = 122;
        let result = esp3_of_enocean_message(&received_message).unwrap();
        assert_eq!(crc_header, result.crc_header);
    }
    #[test]
    fn given_valid_f60201_message_with_valid_header_then_compute_crc_header() {
        // received_message is a valid message from a necklace pushbutton (EEP F6-01-01)
        // Here we test the CRC8H (crc8 computed on header),
        let received_message = vec![
            85, 0, 10, 7, 1, 235, 165, 16, 8, 70, 128, 5, 17, 114, 247, 0, 1, 255, 255, 255, 255,
            65, 0, 235,
        ];
        let result = compute_crc8(&received_message[1..5].to_vec());
        let crc_header: u8 = received_message[5];
        assert_eq!(result, crc_header);
    }

    #[test]
    fn given_valid_f60201_enocean_message_then_return_corresponding_esp() {
        // received_message is a valid message from a necklace pushbutton (EEP -00-01)
        let received_message = vec![
            85, 0, 7, 7, 1, 122, 246, 0, 254, 245, 143, 212, 32, 2, 255, 255, 255, 255, 48, 0, 39,
        ];
        let data_length: u16 = 7;
        let optional_data_length: u8 = 7;
        let packet_type = PacketType::RadioErp1;
        let crc_header: u8 = 122;
        let crc_data: u8 = 39;
        let data: DataType;
        data = DataType::Erp1Data {
            rorg: Rorg::Rps,
            sender_id: [254, 245, 143, 212],
            status: 32,
            payload: [0].to_vec(),
        };

        let opt_data = Some(OptDataType::Erp1OptData {
            subtel_num: 2,
            destination_id: [255, 255, 255, 255],
            rssi: 48,
            security_lvl: 0,
        });
        let esp_packet = ESP3 {
            data_length,
            optional_data_length,
            packet_type: packet_type,
            data,
            opt_data,
            crc_header,
            crc_data,
        };
        let result = esp3_of_enocean_message(&received_message).unwrap();
        assert_eq!(esp_packet, result);
    }

    // Possible errors related tests
    #[test]
    fn given_invalid_encoean_message_with_invalid_crc_data_then_return_error() {
        // received_message is a valid message from a necklace pushbutton (EEP -00-01)
        let invalid_received_message = vec![
            85, 0, 7, 7, 1, 122, 246, 0, 254, 245, 143, 212, 32, 2, 255, 255, 255, 255, 48, 0, 000,
        ];
        assert_eq!(
            esp3_of_enocean_message(&invalid_received_message)
                .unwrap_err()
                .message,
            String::from("CRC Data Error")
        );
    }
    #[test]
    fn given_invalid_a50401_enocean_message_with_no_sync_byte_then_return_error() {
        // received_message is a valid message from a temperature / Humidity sensor (EEP A5-04-01)
        let invalid_received_message = vec![
            54, 0, 10, 7, 1, 235, 165, 16, 8, 70, 128, 5, 17, 114, 247, 0, 1, 255, 255, 255, 255,
            65, 0, 235,
        ];
        assert_eq!(
            esp3_of_enocean_message(&invalid_received_message)
                .unwrap_err()
                .message,
            String::from("Sync Byte Error")
        );
    }
    #[test]
    fn given_invalid_enocean_message_with_invalid_crcheader_then_return_error() {
        // received_message is a valid message from a necklace pushbutton (EEP -00-01) except the 000 CRC8H
        let invalid_received_message = vec![
            85, 0, 7, 7, 1, 000, 246, 0, 254, 245, 143, 212, 32, 2, 255, 255, 255, 255, 48, 0, 39,
        ];
        assert_eq!(
            esp3_of_enocean_message(&invalid_received_message)
                .unwrap_err()
                .message,
            String::from("CRC Error")
        );
    }
    #[test]
    fn given_incomplete_encoean_message_then_return_invalid_input_error() {
        // received_message is a valid message from a necklace pushbutton (EEP -00-01)
        let invalid_received_message = vec![85, 0, 7, 7, 1];
        assert_eq!(
            esp3_of_enocean_message(&invalid_received_message)
                .unwrap_err()
                .message,
            String::from("Invalid input message")
        );
    }

    // Enocean Serial Protocol 3 : ERP1 typical fields
    // -------------------------------------------------------------------
    #[test]
    fn given_valid_a50401_enocean_message_then_return_corresponding_esp() {
        let received_message = vec![
            85, 0, 10, 7, 1, 235, 165, 0, 229, 204, 10, 5, 17, 114, 247, 0, 1, 255, 255, 255, 255,
            54, 0, 213,
        ];
        let esp3_packet: ESP3 = esp3_of_enocean_message(&received_message).unwrap();
        let valid_sender_id: [u8; 4] = [5, 17, 114, 247];
        let valid_payload = vec![0, 229, 204, 10];
        let valid_rorg = Rorg::Bs4;
        let valid_status = 0x00;

        let result_sender_id: [u8; 4];
        let result_rorg: Rorg;
        let result_status: u8;
        let result_payload: Vec<u8>;

        match esp3_packet.data {
            DataType::Erp1Data {
                rorg,
                sender_id,
                status,
                payload,
            } => {
                result_sender_id = sender_id;
                result_rorg = rorg;
                result_status = status;
                result_payload = payload;
            }
            _ => {
                result_sender_id = [0, 1, 2, 3];
                result_rorg = Rorg::Undefined;
                result_status = 0xFF;
                result_payload = vec![0];
            }
        }
        assert_eq!(result_sender_id, valid_sender_id);
        assert_eq!(result_payload, valid_payload);
        assert_eq!(result_rorg, valid_rorg);
        assert_eq!(result_status, valid_status);
    }
    // Enocean Serial Protocol 3 : Response fields
    // -------------------------------------------------------------------
    #[test]
    fn given_valid_response_packet_then_return_corresponding_esp() {
        let header: Vec<u8> = vec![0, 01, 0, 2];
        let crc_header = compute_crc8(&header);
        let data: Vec<u8> = vec![0];
        let crc_data = compute_crc8(&data);

        let mut received_message: Vec<u8> = vec![0x55];
        received_message.extend_from_slice(&header);
        received_message.push(crc_header);
        received_message.extend_from_slice(&data);
        received_message.push(crc_data);

        let esp3_packet: ESP3 = esp3_of_enocean_message(&received_message[..]).unwrap();

        let result_return_code: ReturnCode;
        let result_payload: Option<Vec<u8>>;

        match esp3_packet.data {
            DataType::ResponseData {
                response_payload,
                return_code,
            } => {
                result_return_code = return_code;
                result_payload = response_payload;
            }
            _ => {
                result_return_code = ReturnCode::Undefined;
                result_payload = Some(vec![0, 1, 2, 3]);
            }
        }
        assert_eq!(result_return_code, ReturnCode::Ok);
        assert_eq!(result_payload.is_none(), true);
    }

    // TELEGRAMS examples :
    //
    // A50401 when button is pushed
    // [85, 0, 10, 7, 1, 235, 165, 16, 8, 70, 128, 5, 17, 114, 247, 0, 1, 255, 255, 255, 255, 55, 0, 55]
    //
    // A50401 when button is not pushed (automatic send from trh)
    // [85, 0, 10, 7, 1, 235, 165, 0, 229, 204, 10, 5, 17, 114, 247, 0, 1, 255, 255, 255, 255, 54, 0, 213]

    // F60201 when pushed :
    // [85, 0, 7, 7, 1, 122, 246, 112, 254, 245, 143, 245, 48, 1, 255, 255, 255, 255, 46, 0, 249]
    // [85, 0, 7, 1, 108, 246, 8, 0, 0, 0, 0, 48, 3, 255, 255, 255, 255, 255, 0, 208

    // F60201 when released :
    // [85, 0, 7, 7, 1, 122, 246, 0, 254, 245, 143, 245, 32, 1, 255, 255, 255, 255, 45, 0, 139]

    // Soft Remote NodOn =
    // B1 (cercle plein) pushed
    // [85, 0, 7, 7, 1, 122, 246, 16, 0, 49, 192, 249, 48, 2, 255, 255, 255, 255, 61, 0, 222]

    // B1 (cercle plein) released
    // [85, 0, 7, 7, 1, 122, 246, 0, 0, 49, 192, 249, 32, 1, 255, 255, 255, 255, 55, 0, 114]

    // A0 (-) pushed
    // [85, 0, 7, 7, 1, 122, 246, 48, 0, 49, 192, 249, 48, 1, 255, 255, 255, 255, 51, 0, 144]
    // released
    // [85, 0, 7, 7, 1, 122, 246, 0, 0, 49, 192, 249, 32, 2, 255, 255, 255, 255, 49, 0, 106]

    // Common command : read Base_ID of TCM300
    // CO_RD_IDBASE
    // [85, 0, 5, 1, 2, 219, 0, 255, 155, 18, 128, 10, 17] . BASE ID = 255, 155, 18, 128

}
