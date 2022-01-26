use crate::byte_reader::*;
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
        };
        if protocol_version == 5 {
            // response code
            if length > 0 {
                let reason_code = self.reader.read_u8()?;
                // validate disconnect code
                let reason_code = ReasonCode::validate_disconnect_code(reason_code)?;
            } else {
                packet.reason_code = Some(0);
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
