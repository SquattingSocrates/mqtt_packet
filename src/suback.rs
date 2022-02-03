use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

impl Packet for SubackPacket {
    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<Self> {
        let message_id = reader.read_u16()?;

        let mut packet = SubackPacket {
            reason_code: None,
            properties: None,
            granted_reason_codes: vec![],
            granted_qos: vec![],
            message_id,
        };

        // Properties mqtt 5
        if protocol_version == 5 {
            packet.properties = match reader.read_properties()? {
                None => None,
                Some(props) => Some(ConfirmationProperties::from_properties(props)?),
            };
        }

        if !reader.has_more() {
            return Err("Malformed suback, no payload specified".to_string());
        }

        // Parse granted QoSes
        while reader.has_more() {
            let code = reader.read_u8()?;
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

    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
        // Check message ID
        let mut length = 2;

        // Check granted qos vector
        let granted: Vec<u8> = if protocol_version == 5 {
            self.granted_reason_codes
                .iter()
                .map(|code| code.to_byte())
                .collect()
        } else {
            self.granted_qos.iter().map(|code| code.to_byte()).collect()
        };
        length += granted.len();

        // properies mqtt 5
        let properties_data =
            Properties::encode_option(self.properties.as_ref(), protocol_version)?;
        length += properties_data.len();
        let mut writer = MqttWriter::new(length);
        // header
        writer.write_header(FixedHeader::for_type(PacketType::Suback));

        // Length
        writer.write_variable_num(length as u32)?;

        // Message ID
        writer.write_u16(self.message_id);

        // properies mqtt 5
        writer.write_vec(properties_data);

        // Granted data
        writer.write_vec(granted);
        Ok(writer.into_vec())
    }
}

impl SubackPacket {
    fn new_v3(message_id: u16, granted_qos: Vec<QoS>) -> SubackPacket {
        SubackPacket {
            message_id,
            granted_qos,
            granted_reason_codes: vec![],
            properties: None,
            reason_code: None,
        }
    }
}
