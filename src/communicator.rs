use crate::enocean::*;

use serialport::prelude::*;
use std::time::Duration;

use std::io;
use std::io::Write;

use std::sync::mpsc;

use crate::ParseEspErrorKind;

pub fn start(
    port_name: String,
    enocean_event: mpsc::Sender<ESP3>,
    enocean_command: mpsc::Receiver<ESP3>,
) -> Result<(), std::io::Error>{
    // Set settings as mentioned in ESP3
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(100);
    settings.baud_rate = 57600;
    settings.data_bits = serialport::DataBits::Eight;
    settings.parity = serialport::Parity::None;
    settings.stop_bits = serialport::StopBits::One;
    settings.flow_control = serialport::FlowControl::None;
    // Open serial port (with a sender and receiver)
    // println!("Try to connect to {} : ", port_name);
    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut reader) => {
            let mut serial_buf: Vec<u8> = vec![0; 100];
            let mut incomplete_serial_buf: Option<Vec<u8>> = None;
            // println!("Receiving data on {}:", &port_name);
            // ENOCEAN COMMAND SEND (if any)
            loop {
                let packet_to_send = enocean_command.try_recv();
                match packet_to_send {
                    Ok(packet) => {
                        // println!("sending packet : {:?}", packet);
                        // Convert ESP3 to u8
                        let bytes_to_send = Vec::from(&packet);
                        match reader.write(&bytes_to_send[..]) {
                            Ok(_) => {
                                // print!(".");
                                std::io::stdout().flush().unwrap();
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    Err(_) => {}
                }
                // USB300 MESSAGE RECEIVE (if any)

                match reader.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        // If we received an incomming telegram :
                        // println!("Received telegram : {:X?} ", &serial_buf[..t]);
                        match esp3_of_enocean_message(get_raw_message(serial_buf[..t].to_vec())) {
                            Ok(esp3_packet) => {
                                // If we achieved to transform it into an ESP3 packet, send it to the main thread
                                match enocean_event.send(esp3_packet.clone()) {
                                    Ok(_result) => {}
                                    Err(e) => {
                                        eprintln!(
                                            "Erreur lors de l'envoi du packet : {:?} erreur : {:?}",
                                            esp3_packet, e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                // If message was incomplete, maybe the telegram is just truncated (received in 2 differents parts)
                                match e.kind {
                                    // If it's the "first part"
                                    ParseEspErrorKind::IncompleteMessage => {
                                        // We save it for next incomming telegram parsing
                                        // println!("Saving : {:x?}", e.packet);
                                        incomplete_serial_buf = Some(e.packet);
                                    }
                                    // If it's the "second part"
                                    ParseEspErrorKind::NoSyncByte => {
                                        match incomplete_serial_buf {
                                            // If we have stored the first part before
                                            Some(mut buffer) => {
                                                buffer.extend(e.packet.iter().cloned());
                                                // println!("REPAIRED telegram : {:X?} ", buffer);
                                                match esp3_of_enocean_message(buffer) {
                                                    Ok(esp3_packet) => {
                                                        // send it to the main thread
                                                        match enocean_event
                                                            .send(esp3_packet.clone())
                                                        {
                                                            Ok(_result) => {}
                                                            Err(e) => {
                                                                eprintln!(
                                                            "Erreur lors de l'envoi du packet : {:?} erreur : {:?}",
                                                            esp3_packet, e
                                                            );
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Erreur malgré reconstruction {:?}",
                                                            e
                                                        );
                                                    }
                                                }
                                                incomplete_serial_buf = None;
                                            }
                                            None => {}
                                        }
                                    }
                                    _ => {
                                        eprintln!("Autre erreur : {:?}", e);
                                    }
                                }
                            }
                        }
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => {
                        eprintln!("Error while trying to read serial port input buffer : {:?}", e);
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                     } ,
                }
            } // LOOP END
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            if let Ok(ports) = serialport::available_ports() {
                match ports.len() {
                    0 => println!("No ports found."),
                    1 => println!("Available port :  "),
                    n => println!("Available ports ({}):", n),
                };
                for p in ports {
                    println!("  {}", p.port_name);
                }
            } else {
                print!("Error listing serial ports");
            }
            return Err(std::io::Error::new(std::io::ErrorKind::NotConnected, e.to_string()))
        }
    }
}