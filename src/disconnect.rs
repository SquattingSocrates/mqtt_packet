use crate::packet::*;
use crate::structure::*;
use std::io;

impl<R: io::Read> PacketDecoder<R> {
    pub fn decode_disconnect_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<DisconnectPacket> {
        let mut packet = DisconnectPacket {
            fixed,
            reason_code: None,
            properties: None,
            length,
        };
        if protocol_version == 5 {
            // response code
            if length > 0 {
                let reason_code = self.reader.read_u8()?;
                // validate disconnect code
                let reason_code = DisconnectCode::from_byte(reason_code)?;
                packet.reason_code = Some(reason_code);
            } else {
                packet.reason_code = Some(DisconnectCode::NormalDisconnection);
            }
            // properies mqtt 5
            packet.properties = match self.reader.read_properties()? {
                None => None,
                Some(props) => Some(DisconnectProperties::from_properties(props)?),
            };
        }

        Ok(packet)
    }
}

impl PacketEncoder {
    pub fn encode_disconnect(
        mut self,
        packet: DisconnectPacket,
        protocol_version: u8,
    ) -> Res<Vec<u8>> {
        let mut length = if protocol_version == 5 { 1 } else { 0 };
        // properies mqtt 5
        let properties_data = PropertyEncoder::encode(packet.properties, protocol_version)?;
        length += properties_data.len();
        // Header
        self.write_header(packet.fixed);
        // Length
        self.write_variable_num(length as u32)?;
        // reason code in header
        if protocol_version == 5 && packet.reason_code.is_some() {
            self.write_u8(packet.reason_code.unwrap().to_byte());
        }
        // properies mqtt 5
        self.write_vec(properties_data);
        Ok(self.buf)
    }
}
