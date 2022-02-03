use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::packet::*;
use crate::structure::*;
use std::io;

impl Packet for UnsubackPacket {
    /// This
    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
        // Check message ID
        let mut length = 2;

        // add length of unsubscriptions
        length += self.granted.len();

        // properies mqtt 5
        let properties_data =
            Properties::encode_option(self.properties.as_ref(), protocol_version)?;
        length += properties_data.len();
        let mut writer = MqttWriter::new(length);
        // header
        writer.write_header(FixedHeader::for_type(PacketType::Unsuback));

        // Length
        writer.write_variable_num(length as u32)?;

        // Message ID
        writer.write_u16(self.message_id);

        // properies mqtt 5
        writer.write_vec(properties_data);

        // Granted
        for g in self.granted.iter() {
            writer.write_u8(g.to_byte());
        }
        Ok(writer.into_vec())
    }
    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<Self> {
        let message_id = reader.read_u16()?;

        if (protocol_version == 3 || protocol_version == 4) && length != 2 {
            return Err("Malformed unsuback, payload length must be 2".to_string());
        }
        if length == 0 {
            return Err("Malformed unsuback, no payload specified".to_string());
        }
        let mut packet = UnsubackPacket {
            properties: None,
            granted: vec![],
            message_id,
        };

        // Properties mqtt 5
        if protocol_version == 5 {
            packet.properties = match reader.read_properties()? {
                None => None,
                Some(props) => Some(ConfirmationProperties::from_properties(props)?),
            };
            // Parse granted QoSes

            while reader.has_more() {
                let code = UnsubackCode::from_byte(reader.read_u8()?)?;
                packet.granted.push(code);
            }
        }
        Ok(packet)
    }
}
