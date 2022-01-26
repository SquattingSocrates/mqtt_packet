use crate::packet::*;
use crate::structure::*;
use std::io;

impl PacketEncoder {
    pub fn encode_suback(&mut self, packet: ConfirmationPacket) -> Res<Vec<u8>> {
        Ok(vec![])
    }
}

impl<R: io::Read> PacketDecoder<R> {
    /// Decode SUBACK messages
    pub fn decode_suback_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<SubackPacket> {
        let message_id = self.reader.read_u16()?;

        let mut packet = SubackPacket {
            fixed,
            reason_code: None,
            properties: None,
            granted_reason_codes: vec![],
            granted_qos: vec![],
        };

        // Properties mqtt 5
        if protocol_version == 5 {
            packet.properties = match self.reader.read_properties()? {
                None => None,
                Some(props) => Some(ConfirmationProperties::from_properties(props)?),
            };
        }

        if !self.reader.has_more() {
            return Err("Malformed suback, no payload specified".to_string());
        }

        // Parse granted QoSes
        while self.reader.has_more() {
            let code = self.reader.read_u8()?;
            if protocol_version == 5 {
                packet
                    .granted_reason_codes
                    .push(GrantedReasonCode::from_byte(code)?);
            } else {
                packet.granted_qos.push(match code {
                    0 => QoS::QoS0,
                    1 => QoS::QoS1,
                    2 => QoS::QoS2,
                    _ => return Err("Invalid suback QoS, must be <= 2".to_string()),
                });
            }
        }
        Ok(packet)
    }
}
