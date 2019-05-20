use std::sync::mpsc;
use std::thread;
use std::time::Duration;

extern crate enocean;
use enocean::enocean::*;
use enocean::communicator::EnoceanCommunicator;


fn main() {
    // Just to show how much enocean serial packets were received
    let mut nb_received = 0;
    // Just to show how much enocean serial packets were sended
    let mut nb_sended = 0;
    // For now, this variable is hardcoded
    let port_name = "/dev/tty.usbserial-FT1ZKA73"; // Get this from env?
                                                   // Communication channels based on MPSC (1 to send, 1 to receive esp3 packets)

    // Create a communicator to interact (both ways) with serial port
    // The interaction is achieved thanks to 2 channels (std::sync lib)
    let enocean_communicator = EnoceanCommunicator::new(port_name);
  
    // Create a thread to send commands every second
    // TODO : Change create_xxx_command return type from vecU8 to ESP3 ==> add fromVec trait to ESP3
    // let _command_emiter = thread::spawn(move || loop {
    //     match enocean_communicator.enocean_sender.send(enocean::eep::create_smart_plug_command(
    //         [0x05, 0x0a, 0x3d, 0x6a],
    //         enocean::eep::D201CommandList::QueryPower,
    //     )) {
    //         Ok(_t) => {}
    //         Err(e) => eprintln!("erreur lors de l'envoi : {:?}", e),
    //     }
    //     nb_sended = nb_sended + 1;
    //     println!("---> SENDED : {}", nb_sended);
    //     thread::sleep(Duration::from_millis(10000));
    // });

    // Loop to check if we received something. If everything went well during "send phase",
    // we should receive a Reponse type ESP3 packet with ReturnCode 0 each time we send a command.
    loop {
        let message = enocean_communicator.enocean_receiver.lock().unwrap().try_recv();
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
