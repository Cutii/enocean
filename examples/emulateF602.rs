use std::sync::mpsc;
use std::thread;
use std::time::Duration;

extern crate enocean;

fn main() {
    // Just to show how much enocean serial packets were received
    let mut nb_received = 0;
    
    // For now, this variable is hardcoded
    let port_name = "/dev/tty.usbserial-FTWTOH0A".to_string(); //Get this from env?
                                                               // Communication channels based on MPSC (1 to send, 1 to receive esp3 packets)
    let (enocean_emiter, enocean_event_receiver) = mpsc::channel();
    let (enocean_command_receiver, enocean_commander) = mpsc::channel();

    // Create a thread to interact (both ways) with serial port
    // The interaction is achieved thanks to 2 channels (std::sync lib)
    let _enocean_listener = thread::spawn(move || {
        enocean::communicator::start(port_name, enocean_emiter, enocean_commander);
    });
   let F602_emulate_close = enocean::eep::create_f60201_telegram(
            enocean::eep::F602EmulateCommand::MoveBlindClosed, // QueryEnergy, Off, On
        ).unwrap();
    let F602_emulate_open = enocean::eep::create_f60201_telegram(
            enocean::eep::F602EmulateCommand::MoveBlindOpen, // QueryEnergy, Off, On
        ).unwrap();

    let _command_emiter = thread::spawn(move || loop {
        match enocean_command_receiver.send(F602_emulate_close.clone()) {
            Ok(_t) => {}
            Err(e) => eprintln!("erreur lors de l'envoi : {:?}", e),
        }
        
        thread::sleep(Duration::from_millis(2000));
        match enocean_command_receiver.send(F602_emulate_open.clone()) {
            Ok(_t) => {}
            Err(e) => eprintln!("erreur lors de l'envoi : {:?}", e),
        }
        
        thread::sleep(Duration::from_millis(2000));
        });
    // Loop to check if we received something. If everything went well during "send phase",
    // we should receive a Reponse type ESP3 packet with ReturnCode 0 each time we send a command.
    loop {
        let message = enocean_event_receiver.try_recv();
        match message {
            Ok(esp3_packet) => {
                println!{"Received ESP3 packet : {}", esp3_packet};

                nb_received = nb_received + 1;
                println!("---> RECEIVED : {}", nb_received);
            }
            Err(ref e) if e == &std::sync::mpsc::TryRecvError::Empty => (),
            Err(e) => eprintln!("{:?}", e),
        }
        thread::sleep(Duration::from_millis(10));
    }
}
