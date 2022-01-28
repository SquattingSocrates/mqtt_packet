use crate::packet::*;
use crate::structure::*;
use std::io;

impl PacketEncoder {
    pub fn encode_connack(mut self, packet: ConnackPacket, protocol_version: u8) -> Res<Vec<u8>> {
        let rc = if protocol_version == 5 {
            packet.reason_code
        } else {
            packet.return_code
        };
        let mut length = 2; // length of rc and sessionHeader

        // Check return code
        if rc.is_none() {
            return Err("Invalid return code".to_string());
        }
        let rc = rc.unwrap();
        // mqtt5 properties
        let properties_data = PropertyEncoder::encode(packet.properties, protocol_version)?;
        length += properties_data.len();
        self.buf.push(packet.fixed.encode());
        // length
        let mut length = PacketEncoder::encode_variable_num(length as u32);
        self.buf.append(&mut length);
        self.buf
            .push(if packet.session_present { 0x01 } else { 0x0 });
        self.buf.push(rc);
        self.write_vec(properties_data);
        // return true
        Ok(self.buf)
    }
}

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
        let mut packet = ConnackPacket {
            fixed,
            length,
            session_present: flags == 1,
            ..ConnackPacket::default()
        };

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
