extern crate serialport;

use enocean::enocean::*;
use serialport::prelude::*;
use std::time::Duration;

use std::io;

fn main() {
    // Print available ports
    print_available_ports();

    // Select the USB300 serial port, adapt this to your serial port
    let port_name = "/dev/tty.usbserial-FT1ZKA73".to_string();

    // Set settings as mentioned in ESP3
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(100);
    settings.baud_rate = 57600;
    settings.data_bits = serialport::DataBits::Eight;
    settings.parity = serialport::Parity::None;
    settings.stop_bits = serialport::StopBits::One;
    settings.flow_control = serialport::FlowControl::None;

    // Open serial port
    println!("Try to connect to {}", port_name);
    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("Receiving data on {}:", &port_name);
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        // We received a message
                        println!(" Incomming telegram :{:X?} ", &serial_buf[..t]);
                        // Get-it as enocean_message (byte vector)
                        let enocean_message = get_raw_message(Vec::from(&serial_buf[..t]));
                        // And then parse it
                        match esp3_of_enocean_message(enocean_message) {
                            Ok(u) => {
                                println!("ESP3 PACKET : {:#X?}", u);
                            }
                            Err(e) => {
                                eprintln!("{:?}", e);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}

fn print_available_ports() {
    if let Ok(ports) = serialport::available_ports() {
        match ports.len() {
            0 => println!("No ports found."),
            1 => println!("Found 1 port:"),
            n => println!("Found {} ports:", n),
        };
        for p in ports {
            println!("  {}", p.port_name);
        }
    } else {
        print!("Error listing serial ports");
    }
}
