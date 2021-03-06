use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

impl SubackPacket {
    /// Convenient constructor for v3 SUBACK packet. A v3 SUBACK requires
    /// only a 2 byte message_id and a list of granted QoS for each subscribed
    /// topic/pattern
    /// # Example
    ///
    /// ```
    /// use mqtt_packet_3_5::{SubackPacket, Granted};
    /// let packet = SubackPacket::new_v3(123, vec![Granted::QoS1]);
    /// assert_eq!(packet, SubackPacket {
    ///     message_id: 123,
    ///     granted: vec![Granted::QoS1],
    ///     granted_reason_codes: vec![], // v3 has QoS, v5 has more possible codes
    ///     properties: None,             // v3 has no properties
    ///     reason_code: None,            // v3 has no reason code
    /// });
    ///
    ///
    /// ```
    pub fn new_v3(message_id: u16, granted: Vec<Granted>) -> SubackPacket {
        SubackPacket {
            message_id,
            granted,
            granted_reason_codes: vec![],
            properties: None,
            reason_code: None,
        }
    }
}

impl Packet for SubackPacket {
    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        _: FixedHeader,
        _: u32,
        protocol_version: u8,
    ) -> Res<Self> {
        let message_id = reader.read_u16()?;

        let mut packet = SubackPacket {
            reason_code: None,
            properties: None,
            granted_reason_codes: vec![],
            granted: vec![],
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
                packet.granted.push(Granted::from_byte(code)?);
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
            self.granted.iter().map(|code| code.to_byte()).collect()
        };
        length += granted.len();

        // properies mqtt 5
        let (props_len, properties_data) =
            Properties::encode_option(self.properties.as_ref(), protocol_version)?;
        length += properties_data.len() + props_len.len();
        let mut writer = MqttWriter::new(length);
        // header
        writer.write_header(FixedHeader::for_type(PacketType::Suback));

        // Length
        writer.write_variable_num(length as u32)?;

        // Message ID
        writer.write_u16(self.message_id);

        // properies mqtt 5
        writer.write_sized(&properties_data, &props_len)?;

        // Granted data
        writer.write_vec(granted);
        Ok(writer.into_vec())
    }
}
