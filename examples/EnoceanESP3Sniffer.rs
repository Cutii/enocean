use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

extern crate enocean;
use enocean::eep::*;
use enocean::enocean::*;

fn main() {
    // Just to show how much enocean serial packets were received
    let mut nb_received = 0;

    // For now, this variable is hardcoded
    let port_name = "/dev/tty.usbserial-FT1ZKA73".to_string();

    // Communication channel beetween threads
    let (enocean_emiter, enocean_event_receiver) = mpsc::channel();

    // Non blocking (try_recv() instead of recv())
    loop {
        let message = enocean_event_receiver.try_recv();
        match message {
            Ok(esp3_packet) => {
                // The next function also handle ERP1 (specific type of ESP3 packet wich contain radio telegram from sensor,
                // see Enocean Serial Protocol 3 specification)
                print_esp3(esp3_packet);
                nb_received = nb_received + 1;
                println!("---> RECEIVED : {}", nb_received);
            }
            Err(ref e) if e == &std::sync::mpsc::TryRecvError::Empty => (),
            Err(e) => eprintln!("{:?}", e),
        }
        thread::sleep(Duration::from_millis(10));
    }
}
