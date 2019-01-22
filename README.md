# Enocean implementation for the Rust Programming Language  

Enocean : ([official website](https://www.enocean.com/en/)) is a Radio protocol for Smart Home / Buildings devices.

This lib is a rust implementation of Enocean Serial Protocol, which you can find here: [ESP3](https://www.enocean.com/esp)  
You can use this library with any compatible EnOcean Radio Gateway (eg. [USB300 gateway]https://www.enocean.com/en/enocean-modules/details/usb-300-oem/)).   
  

:warning: **This lib is still under construction** :warning:  

## Feature Overview  
Enocean Radio protocol for Smart Homes rust implementation ([official website](https://www.enocean.com/en/))

 EnOcean is a Radio protocol for SmartHome devices. More informations about EnOcean : [Official website](https://www.enocean.com/en/)
This lib allow you to play with Enocean Serial Protocol, which is defined here: [ESP3](https://www.enocean.com/esp)
You can use this library with any compatible EnOcean Radio Gateway eg. [USB300 gateway](https://www.enocean.com/en/enocean-modules/details/usb-300-oem/).

For now this lib allow you to create an ESP struct from an incomming bytes vector. 

**Supported packet type** :     
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