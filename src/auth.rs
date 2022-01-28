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
            reason_code: AuthCode::Success,
            properties: None,
            length,
        };
        packet.reason_code = AuthCode::from_byte(self.reader.read_u8()?)?;
        // properies mqtt 5
        packet.properties = match self.reader.read_properties()? {
            None => None,
            Some(props) => Some(AuthProperties::from_properties(props)?),
        };
        Ok(packet)
    }
}

impl PacketEncoder {
    /// This
    pub fn encode_auth(mut self, packet: AuthPacket, protocol_version: u8) -> Res<Vec<u8>> {
        if protocol_version != 5 {
            return Err(format!(
                "Invalid mqtt version for auth packet {}",
                protocol_version
            ));
        }
        // Check message ID
        let mut length = 1;

        // properies mqtt 5
        let properties_data = PropertyEncoder::encode(packet.properties, protocol_version)?;
        length += properties_data.len();

        // header
        self.write_header(packet.fixed);

        // Length
        self.write_variable_num(length as u32)?;

        // reason code
        self.write_u8(packet.reason_code.to_byte());

        // properies mqtt 5
        self.write_vec(properties_data);

        Ok(self.buf)
    }
}
