use crate::packet::*;
use crate::structure::*;
use std::io;

impl<R: io::Read> PacketDecoder<R> {
    pub fn decode_auth_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<AuthPacket> {
        if protocol_version != 5 {
            return Err("Not supported auth packet for this version MQTT".to_string());
        }
        // response code
        let mut packet = AuthPacket {
            fixed,
            reason_code: 0,
            properties: None,
        };
        packet.reason_code = ReasonCode::validate_auth_code(self.reader.read_u8()?)?;
        // properies mqtt 5
        packet.properties = match self.reader.read_properties()? {
            None => None,
            Some(props) => Some(AuthProperties::from_properties(props)?),
        };
        Ok(packet)
    }
}
