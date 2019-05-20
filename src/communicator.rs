use crate::enocean::*;

use serialport::prelude::*;
use std::time::Duration;

use std::io;
use std::io::Write;

use crate::ParseEspErrorKind;
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct EnoceanCommunicator {
    pub thread: io::Result<thread::JoinHandle<()>>,
    pub enocean_sender: mpsc::Sender<ESP3>,
    pub enocean_receiver: Arc<Mutex<Receiver<ESP3>>>,
}
impl EnoceanCommunicator {
    /// Given a Board variant, start the SerialPort & (de)serialization thread (1 thread/board) and return its BoardCommunicator
    pub fn new(port_name: &'static str) -> EnoceanCommunicator {
        // SEND COMMANDS OR REQUESTS TO ENOCEAN related variables :
        let (sender, receiver) = mpsc::channel::<ESP3>();
        let receiver = Arc::new(Mutex::new(receiver));
        let enocean_sender = Arc::clone(&receiver);

        // RECEIVE UPDATES / RESPONSES FROM ENOCEAN related variables :
        let (sender_from_enocean, receiver_from_enocean) = mpsc::channel();
        let receiver_from_enocean = Arc::new(Mutex::new(receiver_from_enocean));
        let enocean_receiver = Arc::clone(&receiver_from_enocean);

        // SERIAL PORT
        // Set settings as mentioned in ESP3
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(100);
        settings.baud_rate = 57600;
        settings.data_bits = serialport::DataBits::Eight;
        settings.parity = serialport::Parity::None;
        settings.stop_bits = serialport::StopBits::One;
        settings.flow_control = serialport::FlowControl::None;

        // THREAD START
        let thread = thread::Builder::new()
            .name(String::from(port_name))
            .spawn(move || {
                match serialport::open_with_settings(port_name, &settings) {
                    Ok(mut reader) => {
                        let mut serial_buf: Vec<u8> = vec![0; 100];
                        let mut incomplete_serial_buf: Option<Vec<u8>> = None;

                        // debug!("Receiving data on {}:", &port_name);

                        //-------- -------- -------- MAIN LOOP OF THREAD -------- -------- --------//
                        loop {
                            let packet_to_send = enocean_sender.lock().unwrap().try_recv();
                            match packet_to_send {
                                Ok(packet) => {
                                    println!("sending packet : {:?}", packet);
                                    // TODO : convert from ESP3 to bytes
                                    // match reader.write(&packet[..]) {
                                    //     Ok(_) => {
                                    //         print!(".");
                                    //         std::io::stdout().flush().unwrap();
                                    //     }
                                    //     Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                                    //     Err(e) => eprintln!("{:?}", e),
                                    // }
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
                                            match sender_from_enocean.send(esp3_packet.clone()) {
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
                                                                    match sender_from_enocean
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
                                }
                                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                                Err(e) => eprintln!("{:?}", e),
                            }
                            thread::sleep(Duration::from_millis(1));
                        }
                        //-------- -------- -------- MAIN LOOP OF THREAD END -------- -------- --------//
                    }

                    // If we can't open serial port.
                    Err(e) => {
                        eprintln!("Failed to open port {}. Error: {}", port_name, e);
                    }
                    // Err(e) => {
                    //     // error!("Failed to open port {}. Error: {}", port_name, e);
                    //     // error!("Failed to open port {}. Error: {}", port_name, e);

                    //     if let Ok(ports) = serialport::available_ports() {
                    //         match ports.len() {
                    //             // 0 => debug!("No ports found."),
                    //             // 1 => debug!("Available port :  "),
                    //             // n => debug!("Available ports ({}):", n),
                    //         };
                    //         for p in ports {
                    //             // debug!("  {}", p.port_name);
                    //         }
                    //     } else {
                    //         print!("Error listing serial ports");
                    //         // error!("Error listing serial ports");
                    //     }
                    //     // error!("Unable to start the thread for {}", port_name);
                    //     // panic!("Unable to start the thread");
                    // }
                };
            }); // THREAD END

        EnoceanCommunicator {
            thread: thread,
            enocean_sender: sender,
            enocean_receiver,
        }
    }
}