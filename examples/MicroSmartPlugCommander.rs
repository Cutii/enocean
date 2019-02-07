use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

extern crate enocean;
use enocean::eep::*;
use enocean::enocean::*;

fn main() {
    let mut nb_received = 0;
    let mut nb_sended = 0;

    let port_name = "/dev/tty.usbserial-FT1ZKA73".to_string(); //Get this from env?

    // From enocean thread to this thread
    let (enocean_emiter, enocean_event_receiver) = mpsc::channel();
    let (enocean_command_receiver, enocean_commander) = mpsc::channel();

    let _enocean_listener = thread::spawn(move || {
        enocean::communicator::listen(port_name, enocean_emiter, enocean_commander);
    });

    let command_emiter = thread::spawn(move || loop {
        match enocean_command_receiver.send(enocean::eep::create_smart_plug_command(
            [0x05, 0x0a, 0x3d, 0x6a],
            enocean::eep::D201CommandList::QueryPower,
        )){
            Ok (_t)=>{},
            Err(e)=>{eprintln!("erreur lors de l'envoi : {:?}", e)};
        }
        nb_sended = nb_sended + 1;
        println!("---> SENDED : {}", nb_sended);
        // thread::sleep(Duration::from_millis(1000));
        // enocean_command_receiver.send(enocean::eep::create_smart_plug_command(
        //     [0x05, 0x0a, 0x3d, 0x6a],
        //     enocean::eep::D201CommandList::QueryEnergy,
        // ));
        // nb_sended = nb_sended + 1;
        // println!("---> SENDED : {}", nb_sended);
        // enocean_command_receiver.send(enocean::eep::create_smart_plug_command(
        //     [0x05, 0x0a, 0x3d, 0x6a],
        //     enocean::eep::D201CommandList::Off,
        // ));
        // nb_sended = nb_sended + 1;
        // println!("---> SENDED : {}", nb_sended);
        // enocean_command_receiver.send(enocean::eep::create_smart_plug_command(
        //     [0x05, 0x0a, 0x3d, 0x6a],
        //     enocean::eep::D201CommandList::On,
        // ));
        // nb_sended = nb_sended + 1;
        // println!("---> SENDED : {}", nb_sended);
    });

    loop {
        let message = enocean_event_receiver.try_recv();
        match message {
            Ok(esp3_packet) => {
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
