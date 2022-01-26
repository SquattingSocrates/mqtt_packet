use crate::packet::*;
use crate::structure::*;
use std::io;

impl PacketEncoder {
    fn encode_unsuback(&mut self, packet: UnsubackPacket) -> Res<Vec<u8>> {
        Ok(vec![])
    }
}

impl<R: io::Read> PacketDecoder<R> {
    pub fn decode_unsuback_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<UnsubackPacket> {
        let message_id = self.reader.read_u16()?;

        if (protocol_version == 3 || protocol_version == 4) && length != 2 {
            return Err("Malformed unsuback, payload length must be 2".to_string());
        }
        if !self.reader.has_more() {
            return Err("Malformed unsuback, no payload specified".to_string());
        }
        let mut packet = UnsubackPacket {
            fixed,
            properties: None,
            granted: vec![],
        };

        // Properties mqtt 5
        if protocol_version == 5 {
            packet.properties = match self.reader.read_properties()? {
                None => None,
                Some(props) => Some(ConfirmationProperties::from_properties(props)?),
            };
            // Parse granted QoSes

            while self.reader.has_more() {
                let code = ReasonCode::validate_unsuback_code(self.reader.read_u8()?)?;
                packet.granted.push(code);
            }
        }
        Ok(packet)
    }
}
