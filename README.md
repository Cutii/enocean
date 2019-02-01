# Enocean implementation for the Rust Programming Language           
         
:warning: **This lib is still under construction** :warning:           
         
Enocean : ([official website](https://www.enocean.com/en/)) is a Radio protocol for Smart Home / Buildings devices.         
         
This lib is a rust implementation of Enocean Serial Protocol, which you can find here: [ESP3](https://www.enocean.com/esp)           
You can use this library with any compatible EnOcean Radio Gateway (eg. [USB300 gateway]https://www.enocean.com/en/enocean-modules/details/usb-300-oem/)).           
         
:warning: **This lib is still under construction** :warning:       

## Example    
*cargo run --example main* :    
         
## Feature Overview           
         
This lib use [serialport](https://crates.io/crates/serialport) crate to interact with Serial / Radio gateway.      
:warning: For now, link between enocean device ID and its EEP is hardcoded in eep.rs file.

**Library files main content:** (Non axhaustive, just for quick overview)   
- enocean.rs : Enocean serial protocol implementation (eg . Vector of byte to Ensocean Serial Packet)  (...)   
- commincator.rs : Interface with serialport (use std::sync::mpsc to interact with your code for send /receive packets) (...)     
- eep.rs : Specific for ERP1 packet type, allow to get the content of a radio telegram (...)   
- lib.rs : Custom types / errorTypes (...)   


**Supported Enocean Serial Packets type for now** :              
[x] Radio ERP1 : 0x01             
[x] Response : 0x02              
[ ] radio_sub_tel : 0x03                
[ ] event : 0x04              
[ ] common_command : 0x05             
[ ] smart_ack_command : 0x06             
[ ] remote_man_command : 0x07             
[ ] radio_message : 0x09             
[ ] radio_advanced : 0x0a             
         

## License         
[license]: #license         
         
This library is primarily distributed under the terms of both the MIT license         
and the Apache License (Version 2.0).           
         
See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.         