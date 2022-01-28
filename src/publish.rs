use crate::packet::*;
use crate::structure::*;
use std::io;

impl<R: io::Read> PacketDecoder<R> {
    /// Decode Publish messages
    pub fn decode_publish_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<PublishPacket> {
        let topic = self.reader.read_utf8_string()?;

        let mut packet = PublishPacket {
            fixed,
            topic,
            properties: None,
            payload: vec![],
            message_id: None,
            length,
        };
        // Parse messageId
        if packet.fixed.qos > 0 {
            packet.message_id = Some(self.reader.read_u16()?);
        }

        // Properties mqtt 5
        if protocol_version == 5 {
            packet.properties = match self.reader.read_properties()? {
                None => None,
                Some(props) => Some(PublishProperties::from_properties(props)?),
            };
        }

        packet.payload = self.reader.consume()?;

        Ok(packet)
    }
}

impl PacketEncoder {
    pub fn encode_publish(mut self, packet: PublishPacket, protocol_version: u8) -> Res<Vec<u8>> {
        let mut length = 0;
        let PublishPacket {
            topic,
            fixed,
            message_id,
            properties,
            payload,
            ..
        } = packet;
        let qos = fixed.qos;

        // Topic must be a non-empty string or Buffer
        length += 2 + topic.len();
        if topic.is_empty() {
            return Err("Invalid topic".to_string());
        }

        // Get the payload length
        length += payload.len();

        // Message ID must a number if qos > 0
        if qos > 0 {
            if message_id.is_none() {
                return Err("Invalid messageId".to_string());
            }
            length += 2;
        }

        // mqtt5 properties
        let properties_data = PropertyEncoder::encode(properties, protocol_version)?;
        length += properties_data.len();

        // Header
        self.write_header(fixed);

        // Remaining length
        self.write_variable_num(length as u32)?;

        // Topic
        self.write_utf8_string(topic);

        // Message ID
        if qos > 0 {
            self.write_u16(message_id.unwrap());
        }

        // Properties
        self.write_vec(properties_data);

        // Payload
        self.write_vec(payload);
        Ok(self.buf)
    }
}
