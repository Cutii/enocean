use crate::enocean::*;
use crate::*;
use std::collections::HashMap;

pub fn parse_erp1_payload(esp: &ESP3) -> ParseEspResult<HashMap<String, String>> {
    //
    match &esp.data {
        // ERP Treatments
        DataType::Erp1Data {
            rorg: _rorg,
            sender_id,
            status: _status,
            payload,
        } => {
            match get_eep(sender_id) {
                // The way we parse the packet payload depends on its EEP
                Some(EEP::A50401) => Ok(parse_a50401_data(&payload)),
                Some(EEP::F60201) => Ok(parse_f60201_data(&payload)),
                Some(EEP::F60202) => Ok(parse_f60202_data(&payload)),
                Some(EEP::D2010E) => Ok(parse_d201_data(&payload)),
                Some(EEP::D50001) => Ok(parse_d50001_data(&payload)),

                _ => {
                    return Err(ParseEspError {
                        message: String::from("Unknown EEP"),
                        byte_index: None,
                        packet: Vec::from(esp),
                        kind: ParseEspErrorKind::Unimplemented,
                    })
                }
            }
        }
        _ => Err(ParseEspError {
            message: String::from("Unknown or Unimplemented yet packet type"),
            packet: Vec::from(esp),
            byte_index: Some(6),
            kind: ParseEspErrorKind::Unimplemented,
        }),
    }
}
/// These EEP are currently supported by this lib
pub enum EEP {
    A50401,
    D2010E, //partially supported
    D50001,
    F60201,
    F60202,
}

/// These D201 (eg. smart plugs) commands are supported by this lib
pub enum D201CommandList {
    On,
    Off,
    QueryEnergy,
    QueryPower,
    DefaultConfig,
}
/// These F602 (eg. PTM) messages emulation are supported by this lib
pub enum F602EmulateCommand {
    MoveBlindClosed,
    MoveBlindOpen
}

/// Link between EnOcean ID and EEP. This part has to be improved (stock EEP<->ID somehow)...
pub fn get_eep(id: &[u8; 4]) -> Option<EEP> {
    match id {
        [5, 17, 114, 247] => Some(EEP::A50401),
        [254, 245, 143, 245] => Some(EEP::F60201),
        [0, 49, 192, 249] => Some(EEP::F60202),
        [0x05, 0x0a, 0x3d, 0x6a] => Some(EEP::D2010E),
        [0x01, 0x92, 0x3d, 0xa8] => Some(EEP::D50001),

        _ => None,
    }
}

/// Util : get tha value of a specific bit in a byte
fn bit_of_byte(bit_nb: u8, byte: &u8) -> bool {
    ((byte >> bit_nb) & 1) != 0
}
/// Util : Byte to array of 8 bits conversion
fn bits_of_byte(byte: u8) -> [bool; 8] {
    let mut value: [bool; 8] = [false; 8];
    for i in 0..8 {
        value[7 - i] = bit_of_byte(i as u8, &byte);
    }
    value
}
// ---------------------------------------------------------------------//
// ---------------- Enocean Message parsing ----------------------------//
// ---------------------------------------------------------------------//
/// Specific parsing function for Temperature and humidity sensor
fn parse_a50401_data(payload: &Vec<u8>) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    parsed.insert(String::from("HUM"), format!("{}", payload[1] as f32 * 0.4));
    parsed.insert(
        String::from("TMP"),
        format!("{}", payload[2] as f32 * (40 as f32) / (250 as f32)),
    );
    match bit_of_byte(3, &payload[3]) {
        false => parsed.insert(String::from("LRNB"), String::from("Teach-in telegram")),
        true => parsed.insert(String::from("LRNB"), String::from("Data telegram")),
    };
    match bit_of_byte(1, &payload[3]) {
        false => parsed.insert(
            String::from("TSN"),
            String::from("Temperature sensor not available"),
        ),
        true => parsed.insert(
            String::from("TSN"),
            String::from("Temperature sensor available"),
        ),
    };
    parsed
}
fn parse_d50001_data(payload: &Vec<u8>) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    match bit_of_byte(4, &payload[0]) {
        false => parsed.insert(String::from("LRNB"), String::from("pressed")),
        true => parsed.insert(String::from("LRNB"), String::from("not pressed")),
    };
    match bit_of_byte(7, &payload[0]) {
        false => parsed.insert(String::from("CO"), String::from("open")),
        true => parsed.insert(String::from("CO"), String::from("closed")),
    };
    parsed
}
/// Specific parsing function for pushbutton
fn parse_f60201_data(payload: &Vec<u8>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    match bit_of_byte(3, &payload[0]) {
        false => result.insert(String::from("LRNB"), String::from("Teach-in telegram")),
        true => result.insert(String::from("LRNB"), String::from("Data telegram")),
    };
    match payload[0] {
        0x70 => result.insert(String::from("BTN"), String::from("Pressed")),
        0x00 => result.insert(String::from("BTN"), String::from("Released")),
        _ => result.insert(String::from("BTN"), String::from("Unknown")), //todo : Erreur
    };
    result
}
/// Specific parsing function for soft remote
fn parse_f60202_data(payload: &Vec<u8>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let payload_bits = bits_of_byte(payload[0]);
    match payload_bits[0..3] {
        [false, false, false] => result.insert(String::from("R1"), String::from("A1")),
        [false, false, true] => result.insert(String::from("R1"), String::from("A0")),
        [false, true, false] => result.insert(String::from("R1"), String::from("B1")),
        [false, true, true] => result.insert(String::from("R1"), String::from("B0")),
        _ => result.insert(String::from("R1"), String::from("Unknown")), //todo : Erreur
    };
    match payload_bits[3] {
        false => result.insert(String::from("EB"), String::from("Released")),
        true => result.insert(String::from("EB"), String::from("Pressed")),
    };
    match payload_bits[4..7] {
        [false, false, false] => result.insert(String::from("R2"), String::from("A1")),
        [false, false, true] => result.insert(String::from("R2"), String::from("A0")),
        [false, true, false] => result.insert(String::from("R2"), String::from("B1")),
        [false, true, true] => result.insert(String::from("R2"), String::from("B0")),
        _ => result.insert(String::from("R1"), String::from("Unknown")), //todo : Erreur
    };
    match payload_bits[7] {
        false => result.insert(String::from("SA"), String::from("No 2nd action")),
        true => result.insert(String::from("SA"), String::from("2nd action valid")),
    };
    result
}
/// Specific parsing function for micro smart plug
fn parse_d201_data(payload: &Vec<u8>) -> HashMap<String, String> {
    // First we have to get CMD_ID:
    let command_id: u8 = payload[0] & 0x0f;
    let mut parsed = HashMap::new();

    if command_id == 0x07 {
        let db4_bits = bits_of_byte(payload[1]);
        match db4_bits[0..3] {
            [false, false, false] => parsed.insert(String::from("UN"), String::from("Energy [Ws]")),
            [false, false, true] => parsed.insert(String::from("UN"), String::from("Energy [Wh]")),
            [false, true, false] => parsed.insert(String::from("UN"), String::from("Energy [KWh]")),
            [false, true, true] => parsed.insert(String::from("UN"), String::from("Power[W]")),
            [true, false, false] => parsed.insert(String::from("UN"), String::from("Power[KW]")),
            _ => parsed.insert(String::from("UN"), String::from("Error")), //todo : Erreur
        };

        parsed.insert(String::from("I/O"), format!("{}", payload[1] & 0b00011111));

        // parsed.insert(String::from("MV"),format!("{}", payload[5] +payload[4]<< 8 +payload[3]<< 16 +payload[2]<< 24));
        parsed.insert(
            String::from("MV"),
            format!(
                "{}",
                payload[5] as i32 + payload[4] as i32 * 256 + payload[3] as i32 * 65536
            ),
        );
    } else if command_id == 0x04 {
        let db2_bits = bits_of_byte(payload[0]);
        match db2_bits[0] {
            false => parsed.insert(
                String::from("PF"),
                String::from("Power Failure Detection disabled/not supported"),
            ),
            true => parsed.insert(
                String::from("PF"),
                String::from("Power Failure Detection enabled"),
            ),
        };
        match db2_bits[1] {
            false => parsed.insert(
                String::from("PFD"),
                String::from("Power Failure Detection disabled/not supported"),
            ),
            true => parsed.insert(String::from("PFD"), String::from("Power Failure Detected")),
        };
        // ... insert here missing EEP fields
        match payload[2] & 0b01111111 {
            0x00 => parsed.insert(String::from("OV"), String::from("Output value : 0% or OFF")),
            0x7F => parsed.insert(
                String::from("OV"),
                String::from("Output value : 1 to 100% or ON"),
            ),
            0x01...0x64 => parsed.insert(String::from("OV"), String::from("Not used")),
            0x65...0x7E => parsed.insert(
                String::from("OV"),
                String::from("Output value not valid / not set"),
            ),
            _ => parsed.insert(String::from("OV"), String::from("Error")),
        };
    } else {
        parsed.insert(String::from("Error"), String::from("Bad CMD ID"));
    }
    parsed
}

// ------------------------------------------------------------------------//
// ---------------- Enocean Message Generation ----------------------------//
// ------------------------------------------------------------------------//
/// Generic message 
pub fn create_f60201_telegram(command: F602EmulateCommand)->ParseEspResult<ESP3> {
    let mut packet: Vec<u8> = vec![0x55];
    let usb_gw_id: Vec<u8> = vec![0, 0, 0, 0];
    let mut data: Vec<u8> = Vec::new();
    
    data.push(0xf6); // choice
    match command {
        F602EmulateCommand::MoveBlindClosed => {
            data.extend_from_slice(&[0x10]); 
        },
        F602EmulateCommand::MoveBlindOpen =>{
            data.extend_from_slice(&[0x30]);      
        }
    }
    data.extend_from_slice(&usb_gw_id);
    data.push(0x30); //status T21 NU to 1 
    let data_length: u8 = data.len() as u8;

    // OPT_DATA
    let mut opt_data: Vec<u8> = vec![0x03];
    opt_data.extend_from_slice(&[0xff,0xff,0xff,0xff]);
    opt_data.push(0xff);
    opt_data.push(0x00);
    let opt_len: u8 = opt_data.len() as u8;

    // HEADER
    let mut header: Vec<u8> = Vec::new();
    header.push(0x00); //data length MSB
    header.push(opt_len);
    header.push(data_length);
    header.push(0x01); //packet type radio

    // CRCs
    let crc_header = compute_crc8(&header);
    println!("{}",crc_header);
    data.append(&mut opt_data);
    let crc_data = compute_crc8(&data);
    println!("{}",crc_data);

    packet.extend_from_slice(&header);
    packet.push(crc_header);
    packet.extend_from_slice(&data);
    packet.extend_from_slice(&opt_data);
    packet.push(crc_data);
    esp3_of_enocean_message(packet)
}

/// UTE telegram acceptation
pub fn create_smart_plug_teach_in_accepted_response_packet(socket_id: [u8; 4]) -> ParseEspResult<ESP3> {
    // Data
    let rorg = 0xd4;
    // let bidirectional_comm = [0,1];
    // let reponse_code= [0,1] ; //teachin accepted
    let infos = 0xd1;
    // let infos = 0xd1;
    let mut mimic: Vec<u8> = vec![1, 70, 0, 14, 1, 210];
    let mut usb_gw_id: Vec<u8> = vec![0, 0, 0, 0];
    // let mut usb_gw_id: Vec<u8> = vec![255, 155, 18, 128];
    let last: u8 = 0;

    let mut data: Vec<u8> = Vec::new();

    data.push(rorg);
    data.push(infos);
    data.append(&mut mimic);
    data.append(&mut usb_gw_id);
    data.push(last);
    // println!("DATA : {:#x?}", data);

    //Opt data
    let send_flag: u8 = 0x03;
    let dbm: u8 = 255;
    let security: u8 = 0;

    let mut opt_data: Vec<u8> = Vec::new();
    opt_data.push(send_flag);
    opt_data.extend_from_slice(&socket_id);
    opt_data.push(dbm);
    opt_data.push(security);
    // println!("OPT_DATA : {:#x?}", opt_data);

    let data_length: u8 = data.len() as u8;
    let opt_len: u8 = opt_data.len() as u8;

    data.append(&mut opt_data);

    //Let's construct the packet
    let crc_data = compute_crc8(&data);

    let packet_type: u8 = 0x01;
    let mut header: Vec<u8> = Vec::new();
    header.push(0x00); //data length= 16 bits)
    header.push(data_length);
    header.push(opt_len);
    header.push(packet_type);
    // println!("HEADER : {:#x?}", header);

    let crc_header = compute_crc8(&header);

    let mut esp3_packet: Vec<u8> = vec![0x55];
    esp3_packet.append(&mut header);
    esp3_packet.push(crc_header);
    esp3_packet.append(&mut data);
    esp3_packet.append(&mut opt_data);
    esp3_packet.push(crc_data);
    // println!("PACKET : {:#x?}", esp3_packet);
    esp3_of_enocean_message(esp3_packet)
}
/// SmartPLug commands creation
pub fn create_smart_plug_command(socket_id: [u8; 4], command: D201CommandList) -> ParseEspResult<ESP3> {
    let mut packet: Vec<u8> = vec![0x55];
    let mut usb_gw_id: Vec<u8> = vec![0, 0, 0, 0];
    let mut data: Vec<u8> = Vec::new();
    match command {
        D201CommandList::Off => {
            data.extend_from_slice(&[0xd2, 0x01, 0x00, 0x00]); // 01 = CMD ID // 00 00 = output 0 to 0
        }
        D201CommandList::On => {
            data.extend_from_slice(&[0xd2, 0x01, 0x00, 0x01]); // 01 = CMD ID // 00 00 = output 0 to 1
        }
        D201CommandList::QueryEnergy => {
            data.extend_from_slice(&[0xd2, 0x06, 0x00]); // 06 = CMD ID // query Energy (Default config = Wh)
        }
        D201CommandList::QueryPower => {
            data.extend_from_slice(&[0xd2, 0x06, 0x20]); // 06 = CMD ID // query power (Default Config = W)
        }
        D201CommandList::DefaultConfig => {
            let db_4: u8 = 0b10100000; // b0: autoreporting , b1 : no reset, b2 : power measurement, then channel nb (0)
            let db_3: u8 = 0x33; // B0-3 = report delta 3w, b4-7: unit = watts
            let db_2: u8 = 0x00; // MSB of report delta, 0 here
            let db_1: u8 = 0x06; // max time between 2 messages = 6 * 10 secondes
            let db_0: u8 = 0x01; // min time between 2 messages = 1 * 1 second

            // DATA
            let mut data: Vec<u8> = vec![0xd2, 0x05]; // 05 = CMD ID
            data.push(db_4);
            data.push(db_3);
            data.push(db_2);
            data.push(db_1);
            data.push(db_0);
        }
    }
    //DATA
    data.append(&mut usb_gw_id);
    data.push(0x00);
    let data_length: u8 = data.len() as u8;
    // OPT_DATA
    let mut opt_data: Vec<u8> = vec![0x03];
    opt_data.extend_from_slice(&socket_id);
    opt_data.push(0xff);
    opt_data.push(0x00);
    let opt_len: u8 = opt_data.len() as u8;

    // HEADER
    let mut header: Vec<u8> = Vec::new();
    header.push(0x00); //data length= 16 bits)
    header.push(data_length);
    header.push(opt_len);
    header.push(0x01); //packet type radio

    // CRCs
    let crc_header = compute_crc8(&header);
    data.append(&mut opt_data);
    let crc_data = compute_crc8(&data);

    packet.extend_from_slice(&header);
    packet.push(crc_header);
    packet.extend_from_slice(&data);
    packet.extend_from_slice(&opt_data);
    packet.push(crc_data);
    esp3_of_enocean_message(packet)
}
/// Config a D2010E micro smart plug 
pub fn create_smart_plug_default_config_packet(socket_id: [u8; 4]) -> ParseEspResult<ESP3>{
    let mut result: Vec<u8> = vec![0x55];
    let mut usb_gw_id: Vec<u8> = vec![0, 0, 0, 0];

    let db_4: u8 = 0b10100000; // b0: autoreporting , b1 : no reset, b2 : power measurement, then channel nb (0)
    let db_3: u8 = 0x33; // B0-3 = report delta 3w, b4-7: unit = watts
    let db_2: u8 = 0x00; // MSB of report delta, 0 here
    let db_1: u8 = 0x06; // max time between 2 messages = 6 * 10 secondes
    let db_0: u8 = 0x01; // min time between 2 messages = 1 * 1 second

    // DATA
    let mut data: Vec<u8> = vec![0xd2, 0x05]; // 05 = CMD ID
    data.push(db_4);
    data.push(db_3);
    data.push(db_2);
    data.push(db_1);
    data.push(db_0);
    data.append(&mut usb_gw_id);
    data.push(0x00); //status

    let data_length: u8 = data.len() as u8;

    // OPT_DATA
    let mut opt_data: Vec<u8> = vec![0x03];
    opt_data.extend_from_slice(&socket_id);
    opt_data.push(0xff);
    opt_data.push(0x00);
    let opt_len: u8 = opt_data.len() as u8;

    // HEADER
    let mut header: Vec<u8> = Vec::new();
    header.push(0x00); //data length= 16 bits)
    header.push(data_length);
    header.push(opt_len);
    header.push(0x01); //packet type radio

    // CRCs
    let crc_header = compute_crc8(&header);
    data.append(&mut opt_data);
    let crc_data = compute_crc8(&data);

    result.append(&mut header);
    result.push(crc_header);
    result.append(&mut data);
    result.append(&mut opt_data);
    result.push(crc_data);

    esp3_of_enocean_message(result)
}

/// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    // ESP3 - ERP1 - EEP specified fields PARSING
    // --------------------------------------------------------------------
    #[test]
    fn given_valid_a50401_esp3_packet_and_its_eep_then_parse_all_data_when_learn_button_not_pressed(
    ) {
        let received_message = vec![
            85, 0, 10, 7, 1, 235, 165, 0, 229, 204, 10, 5, 17, 114, 247, 0, 1, 255, 255, 255, 255,
            54, 0, 213,
        ];
        let _result_sender_id: &[u8; 4];
        let _result_rorg: &Rorg;
        let _result_status: &u8;
        let _result_payload: Vec<u8>;

        let esp3_packet = esp3_of_enocean_message(received_message).unwrap();

        let _eep: EEP = EEP::A50401;

        let results = parse_erp1_payload(&esp3_packet);
        let temp = results.unwrap();
        assert_eq!(temp.get("HUM").unwrap(), &String::from("91.6"));
        assert_eq!(temp.get("TMP").unwrap(), &String::from("32.64"));
        assert_eq!(temp.get("LRNB").unwrap(), &String::from("Data telegram"));
        assert_eq!(
            temp.get("TSN").unwrap(),
            &String::from("Temperature sensor available")
        );
    }
    #[test]
    fn given_valid_f60201_esp3_packet_when_pressed_then_parse_all_data() {
        let received_message = vec![
            85, 0, 7, 7, 1, 122, 246, 112, 254, 245, 143, 245, 48, 1, 255, 255, 255, 255, 46, 0,
            249,
        ];
        let esp3_packet = esp3_of_enocean_message(received_message).unwrap();
        let _eep: EEP = EEP::F60201;

        let results = parse_erp1_payload(&esp3_packet).unwrap();

        assert_eq!(results.get("BTN").unwrap(), &String::from("Pressed"));
    }

    #[test]
    fn given_valid_f60202_esp3_packet_when_a0_pushed_then_parse_all_data() {
        let received_message = vec![
            85, 0, 7, 7, 1, 122, 246, 48, 0, 49, 192, 249, 48, 1, 255, 255, 255, 255, 51, 0, 144,
        ];
        let esp3_packet = esp3_of_enocean_message(received_message).unwrap();
        let results = parse_erp1_payload(&esp3_packet).unwrap();

        assert_eq!(results.get("R1").unwrap(), &String::from("A0"));
    }

    #[test]
    fn given_valid_d2010e_esp3_packet_when_consumption_changes_then_parse_all_data() {
        let received_message = vec![
            0x55, 0x0, 0xC, 0x7, 0x1, 0x96, 0xD2, 0x7, 0x60, 0x0, 0x0, 0x0, 0x13, 0x5, 0xA, 0x3D,
            0x6A, 0x0, 0x1, 0xFF, 0xFF, 0xFF, 0xFF, 0x3D, 0x0, 0xF1,
        ];

        let esp3_packet = esp3_of_enocean_message(received_message).unwrap();
        let results = parse_erp1_payload(&esp3_packet).unwrap();
        assert_eq!(results.get("MV").unwrap(), &String::from("19"));
        assert_eq!(results.get("UN").unwrap(), &String::from("Power[W]"));
    }
    // ESP3 - ERP1 - EEP specified fields EMULATION
    // --------------------------------------------------------------------
    #[test]
    fn given_f60201_valid_status_then_create_valid_f60201_packet() {
        let created_response_close =
            create_f60201_telegram(F602EmulateCommand::MoveBlindClosed).unwrap();
        let valid_response_close = vec![
            0x55, 0x0, 0x07, 0x7, 0x1, 122, 
            0xf6, 0x08, 0x00,0x00,0x00,0x00,0x30, 
            0x03, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0, 208
        ];
        
        assert_eq!(valid_response_close, Vec::from(&created_response_close));
    }

    // UTE TeachIn Payload parsing // response (brut version)
    // --------------------------------------------------------------------
    #[test]
    fn given_d2010e_valid_esp3_packet_containing_teachin_query_then_create_valid_response_packet() {
        let created_response =
            create_smart_plug_teach_in_accepted_response_packet([0x05, 0x0a, 0x3d, 0x6a]).unwrap();
        let valid_response = vec![
            0x55, 0x0, 0xd, 0x7, 0x1, 0xfd, 0xd4, 0xd1, 0x1, 0x46, 0x0, 0xe, 0x1, 0xd2, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x3, 0x5, 0xa, 0x3d, 0x6a, 0xff, 0x0, 0x6d,
        ];
        assert_eq!(valid_response, Vec::from(&created_response));
    }

    // Testing some util fn
    // --------------------------------------------------------------------
    #[test]
    fn given_u8_byte_then_get_specific_bit_value() {
        let a: u8 = 0xa5;
        assert_eq!(bit_of_byte(0, &a), true);
        assert_eq!(bit_of_byte(1, &a), false);
        assert_eq!(bit_of_byte(2, &a), true);
        assert_eq!(bit_of_byte(3, &a), false);
        assert_eq!(bit_of_byte(4, &a), false);
        assert_eq!(bit_of_byte(5, &a), true);
        assert_eq!(bit_of_byte(6, &a), false);
        assert_eq!(bit_of_byte(7, &a), true);
    }

    #[test]
    fn given_u8_byte_then_get_bits_values() {
        let a: u8 = 0xff;
        let b: u8 = 0x00;
        let c: u8 = 0x3a;

        assert_eq!(
            bits_of_byte(a),
            [true, true, true, true, true, true, true, true]
        );
        assert_eq!(
            bits_of_byte(b),
            [false, false, false, false, false, false, false, false]
        );
        assert_eq!(
            bits_of_byte(c),
            [false, false, true, true, true, false, true, false]
        );
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

    // D2010E automatic report (power consumption change > threshold)
    // [55, 0, C, 7, 1, 96, D2, 7, 60, 0, 0, 0, 13, 5, A, 3D, 6A, 0, 1, FF, FF, FF, FF, 3D, 0, F1]

    // Common command : read Base_ID of TCM300
    // CO_RD_IDBASE
    // [85, 0, 5, 1, 2, 219, 0, 255, 155, 18, 128, 10, 17] . BASE ID = 255, 155, 18, 128

}
