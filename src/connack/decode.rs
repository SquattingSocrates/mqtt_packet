use crate::byte_reader::*;
use crate::packet::*;
use crate::structure::*;
use std::io;

impl<R: io::Read> PacketDecoder<R> {
    pub fn decode_connack_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<ConnackPacket> {
        let flags = self.reader.read_u8()?;
        if flags > 1 {
            return Err("Invalid connack flags, bits 7-1 must be set to 0".to_string());
        }
        let mut packet = ConnackPacket::default();
        packet.fixed = fixed;
        packet.length = length;
        packet.session_present = flags == 1;
        println!("PACKET {:?}", packet);

        if protocol_version == 5 {
            packet.reason_code = if length >= 2 {
                Some(self.reader.read_u8()?)
            } else {
                Some(0)
            };
        } else {
            if length < 2 {
                return Err("Packet too short".to_string());
            }
            packet.return_code = Some(self.reader.read_u8()?);
        }
        // mqtt 5 properties
        if protocol_version == 5 && self.reader.has_more() {
            packet.properties = match self.reader.read_properties()? {
                None => None,
                Some(props) => Some(ConnackProperties::from_properties(props)?),
            };
        }
        Ok(packet)
    }
}
