use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

impl Packet for PublishPacket {
    /// Decode Publish messages
    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<PublishPacket> {
        let topic = reader.read_utf8_string()?;

        let mut packet = PublishPacket {
            dup: fixed.dup,
            qos: fixed.qos,
            retain: fixed.retain,
            topic,
            properties: None,
            payload: vec![],
            message_id: None,
        };
        // Parse messageId
        if fixed.qos > 0 {
            packet.message_id = Some(reader.read_u16()?);
        }

        // Properties mqtt 5
        if protocol_version == 5 {
            packet.properties = match reader.read_properties()? {
                None => None,
                Some(props) => Some(PublishProperties::from_properties(props)?),
            };
        }

        packet.payload = reader.consume()?;

        Ok(packet)
    }

    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
        let mut length = 0;
        let PublishPacket {
            topic,
            qos,
            message_id,
            properties,
            payload,
            dup,
            retain,
        } = &self;

        // Topic must be a non-empty string or Buffer
        length += 2 + topic.len();
        if topic.is_empty() {
            return Err("Invalid topic".to_string());
        }

        // Get the payload length
        length += payload.len();

        // Message ID must a number if qos > 0
        if *qos > 0 {
            if message_id.is_none() {
                return Err("Invalid messageId".to_string());
            }
            length += 2;
        }

        // mqtt5 properties
        let properties_data = Properties::encode_option(properties.as_ref(), protocol_version)?;
        length += properties_data.len();
        let mut writer = MqttWriter::new(length);
        // Header
        writer.write_header(FixedHeader {
            cmd: PacketType::Publish,
            qos: *qos,
            dup: *dup,
            retain: *retain,
        });

        // Remaining length
        writer.write_variable_num(length as u32)?;

        // Topic
        writer.write_utf8_str(topic);

        // Message ID
        if *qos > 0 {
            writer.write_u16(message_id.unwrap());
        }

        // Properties
        writer.write_vec(properties_data);

        // Payload
        writer.write_slice(payload);
        Ok(writer.into_vec())
    }
}
