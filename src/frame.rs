
//! # EnOcean ESP3 frames
//! 
//! A low-level interface to ESP3 frames. 
//! 
//! This module will handle frame synchronization, memory allocation, and CRC checks.
//! 
//! Data, optional data, and packet type are exposed as raw values and byte slices.

use std::borrow::Borrow;

use crate::FrameReadError;
use crate::crc8::{compute_crc8, CRC8};

/// An owned ESP3 frame that has been CRC-checked. Includes synchronization byte and CRCs.
#[derive(Clone, Debug)]
pub struct ESP3Frame {
    packet_type: u8,
    data_length: usize,
    optional_data_length: usize,
    frame: Vec<u8>
}

/// Borrowed contents of an ESP3 frame. Can also be used to assemble a new frame.
pub struct ESP3FrameRef<'a> {
    pub packet_type: u8,
    pub data: &'a [u8],
    pub optional_data: &'a [u8],
}

impl ESP3Frame {

    /// Read a frame from a buffered reader. Will perform header synchronization. Allocates exactly the space needed. 
    pub fn read(reader: &mut impl std::io::BufRead) -> Result<Self, FrameReadError> {
      
        let header = loop {  // Synchronize with start of packet
            let buf = reader.fill_buf()?;
            if buf.len() == 0 { return Err(FrameReadError::EOF) }
            if buf[0] != 0x55 {  // Look for synchronizatin byte
                reader.consume(1);
                continue;
            }

            if buf.len() < 6 { continue; }

            if compute_crc8(&buf[1..6]) != 0 {  // Check header CRC. If it fails, keep looking for another sync byte.
                reader.consume(1);
                continue;
            }
            
            break buf;
        };

        // The frame is now synchronized and the header CRC is valid
        // decode the header
        let data_length = ((header[1] as usize) << 8) + (header[2] as usize);
        let optional_data_length = header[3] as usize;
        let packet_type = header[4];
        
        // Allocate an appropriate buffer
        let total_length = 6 + data_length + optional_data_length + 1;
        let mut frame = vec![0; total_length];

        reader.read_exact(&mut frame)?;

        // Check the Data CRC
        let data_crc = compute_crc8(&frame[6..]);
        if data_crc != 0 { return Err(FrameReadError::DataCRC{ frame, data_crc }) }

        Ok(ESP3Frame { frame, packet_type, data_length, optional_data_length })

    }

    /// The packet type, as a single byte
    pub fn packet_type(&self) -> u8 {
        self.packet_type
    }

    /// The frame mandatory, fixed-format data
    pub fn data(&self) -> &[u8] {
        &self.frame[6..][..self.data_length]
    }

    /// The optional data
    pub fn optional_data(&self) -> &[u8] {
        &self.frame[6+self.data_length..][..self.optional_data_length]    
    }
    
    /// Borrows an ESP3Frame as an ESPFrameRef
    pub fn as_ref(&self) -> ESP3FrameRef {
        ESP3FrameRef { packet_type: self.packet_type
                     , data: self.data()
                     , optional_data: self.optional_data()
                     }
    }

    /// Writes the complete frame
    pub fn write(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error> {
        writer.write_all(&self.frame)
    }
}

impl Borrow<[u8]> for ESP3Frame {
    fn borrow(&self) -> &[u8] {
        &self.frame
    }
}

impl<'a> From<ESP3FrameRef<'a>> for ESP3Frame {
    fn from(value: ESP3FrameRef<'a>) -> Self { value.to_owned() }
}

impl<'a> ESP3FrameRef<'a> {

    /// Generate and write a frame
    pub fn write(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error> {

        // Build the header
        let data_len = self.data.len() as u16;
        let data_high = (data_len >> 8) as u8;
        let data_low = (data_len & 0xff) as u8;
        let opt_len = self.optional_data.len() as u8;

        let mut header = [0x55, data_high, data_low, opt_len, self.packet_type, 0];

        // CRC the header
        header[5] = CRC8::from(&header[..5]).into();
        writer.write_all(&header[..])?;

        // CRC the payload
        let data_crc = CRC8::from(self.data).extend(self.optional_data).into();

        // Build the payload
        writer.write_all(self.data)?;
        writer.write_all(self.optional_data)?;
        writer.write_all(&[data_crc])

    }

    // Copies the pieces of a constructed ESP3FrameRef into a single-buffer owned ESP3Frame
    pub fn to_owned(&self) -> ESP3Frame {
        let mut frame = Vec::with_capacity(6 + self.data.len() + self.optional_data.len() + 1);
        self.write(&mut frame).unwrap();   // Writing to a preallocated Vec should never fail

        ESP3Frame { packet_type: self.packet_type, 
                    data_length: self.data.len(), 
                    optional_data_length: self.optional_data.len(),
                    frame }

    }

}