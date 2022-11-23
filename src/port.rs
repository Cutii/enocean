//! Stateful link to an ESP3 device

use serialport::{self, SerialPort};
use std::collections::VecDeque;

use crate::{frame::{ESP3Frame, ESP3FrameRef}, FrameReadError, packet::{Packet, CommonCommand, Response, VersionResponse}, PacketError};

/// An opened ESP3 device.
pub struct Port {
    port: Box<dyn SerialPort>,

    /// In the future, this should store pending requests so that we can route the responses to the correct sender.
    queue: VecDeque<ESP3Frame>
}

impl Port {

    pub fn open_default() -> Result<Self, serialport::Error> {
        todo!()
    }

    pub fn open(port_name: &str) -> Result<Self, serialport::Error> {
        let baud_rate = 57600;
        let port = serialport::new(port_name, baud_rate)
            //.timeout(Duration::from_millis(100))
            .data_bits(serialport::DataBits::Eight)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .flow_control(serialport::FlowControl::None)
            .open()?;

        let queue = VecDeque::new();

        Ok(Self { port, queue })
    }

    pub fn read_version_information(&mut self) -> Result<VersionResponse, PacketError> {
        let response = self.write_packet(Packet::CommonCommand(CommonCommand::ReadVersion))?;
        Ok(VersionResponse::decode(&response)?)
    }

    /// Read the next frame from the port.
    pub fn read_frame(&mut self) -> Result<ESP3Frame, FrameReadError> {
        ESP3Frame::read_from(&mut self.port)
    }

    /// Write a frame to the port.
    pub fn write_frame(&mut self, frame: &ESP3Frame) -> Result<(), std::io::Error> {
        frame.write_to(&mut self.port)
    }

    /// Write a frame to the port.
    /// 
    /// This performs a vectored write.
    /// If you already have a `&EPS3Frame`, use `write_frame` instead.
    pub fn write_frame_ref(&mut self, frame: ESP3FrameRef) -> Result<(), std::io::Error> {
        frame.write_to(&mut self.port)
    }

    pub fn write_packet(&mut self, packet: Packet) -> Result<Response, PacketError> {
        let frame = packet.encode();
        self.write_frame(&frame)?;

        let reply = loop {
            let frame = self.read_frame()?;
            if frame.packet_type() != 0x02 {
                self.queue.push_back(frame);
            } else {
                break frame;
            }
        };

        Ok(Response::decode(reply.as_ref())?)

    }

}
