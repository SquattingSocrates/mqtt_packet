use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

impl Packet for AuthPacket {
    /// This
    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
        if protocol_version != 5 {
            return Err(format!(
                "Invalid mqtt version for auth packet {}",
                protocol_version
            ));
        }
        // Check message ID
        let mut length = 1;

        // properies mqtt 5
        let (props_len, properties_data) =
            Properties::encode_option(self.properties.as_ref(), protocol_version)?;
        length += properties_data.len() + props_len.len();
        let mut writer = MqttWriter::new(length);
        // header
        writer.write_header(FixedHeader::for_type(PacketType::Auth));

        // Length
        writer.write_variable_num(length as u32)?;

        // reason code
        writer.write_u8(self.reason_code.to_byte());

        // properies mqtt 5
        writer.write_sized(&properties_data, &props_len)?;

        Ok(writer.into_vec())
    }

    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        _: FixedHeader,
        _: u32,
        protocol_version: u8,
    ) -> Res<AuthPacket> {
        if protocol_version != 5 {
            return Err("Not supported auth packet for this version MQTT".to_string());
        }
        // response code
        let mut packet = AuthPacket {
            reason_code: AuthCode::Success,
            properties: None,
        };
        packet.reason_code = AuthCode::from_byte(reader.read_u8()?)?;
        // properies mqtt 5
        packet.properties = match reader.read_properties()? {
            None => None,
            Some(props) => Some(AuthProperties::from_properties(props)?),
        };
        Ok(packet)
    }
}
