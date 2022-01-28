use crate::packet::*;
use crate::structure::*;
use std::io;

impl PacketEncoder {
    pub fn encode_suback(mut self, packet: SubackPacket, protocol_version: u8) -> Res<Vec<u8>> {
        // Check message ID
        let mut length = 2;

        // Check granted qos vector
        let granted: Vec<u8> = if protocol_version == 5 {
            packet
                .granted_reason_codes
                .iter()
                .map(|code| code.to_byte())
                .collect()
        } else {
            packet
                .granted_qos
                .iter()
                .map(|code| code.to_byte())
                .collect()
        };
        length += granted.len();

        // properies mqtt 5
        let properties_data = PropertyEncoder::encode(packet.properties, protocol_version)?;
        length += properties_data.len();

        // header
        self.write_header(packet.fixed);

        // Length
        self.write_variable_num(length as u32)?;

        // Message ID
        self.write_u16(packet.message_id);

        // properies mqtt 5
        self.write_vec(properties_data);

        // Granted data
        self.write_vec(granted);
        Ok(self.buf)
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
            message_id,
            length,
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
                    .push(SubscriptionReasonCode::from_byte(code)?);
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
