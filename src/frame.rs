use std::io::IoSlice;
use std::ops::Deref;

use crate::PacketReadError;
use crate::crc8::{compute_crc8, CRC8};

/// An ESP3 frame that has been CRC-checked
pub struct ESP3Frame {
    packet_type: u8,
    data_length: usize,
    optional_data_length: usize,
    frame: Vec<u8>
}

pub struct ESP3FrameRef<'a> {
    pub packet_type: u8,
    pub data: &'a [u8],
    pub optional_data: &'a [u8],
}

impl ESP3Frame {

    pub fn read(reader: &mut impl std::io::BufRead) -> Result<Self, PacketReadError> {
      
        let header = loop {  // Synchronize with start of packet
            let buf = reader.fill_buf()?;
            if buf.len() == 0 { return Err(PacketReadError::EOF) }
            if buf[0] != 0x55 {
                reader.consume(1);
                continue;
            }

            if buf.len() < 6 { continue; }

            if compute_crc8(&buf[1..6]) != 0 {
                reader.consume(1);
                continue;
            }
            
            break buf;
        };

        let data_length = ((header[1] as usize) << 8) + (header[2] as usize);
        let optional_data_length = header[3] as usize;
        let packet_type = header[4];
        
        let total_length = 6 + data_length + optional_data_length + 1;
        let mut frame = vec![0; total_length];

        reader.read_exact(&mut frame)?;

        let data_crc = compute_crc8(&frame[6..]);
        if data_crc != 0 { return Err(PacketReadError::DataCRC(data_crc)) }

        Ok(ESP3Frame { frame, packet_type, data_length, optional_data_length })

    }

    pub fn as_ref(&self) -> ESP3FrameRef {
        ESP3FrameRef { packet_type: self.packet_type
                     , data: &self.frame[6..][..self.data_length]
                     , optional_data: &self.frame[6+self.data_length..][..self.optional_data_length]
                    }
    }

    pub fn write(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error> {
        writer.write_all(&self.frame)
    }
}

impl<'a> ESP3FrameRef<'a> {
    pub fn write(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error> {

        let data_len = self.data.len() as u16;
        let data_high = (data_len >> 8) as u8;
        let data_low = (data_len & 0xff) as u8;
        let opt_len = self.optional_data.len() as u8;

        let header = [0x55, data_high, data_low, opt_len, self.packet_type, 0];

        header[5] = CRC8::from(&header[..5]).into();

        writer.write_all(&header[..])?;
        writer.write_all(&self.data)?;
        writer.write_all(&self.optional_data)

    }
}