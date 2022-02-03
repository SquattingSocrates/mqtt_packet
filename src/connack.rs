use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

impl Packet for ConnackPacket {
    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
        let rc = if protocol_version == 5 {
            self.reason_code
        } else {
            self.return_code
        };
        let mut length = 2; // length of rc and sessionHeader
                            // Check return code
        if rc.is_none() {
            return Err("Invalid return code".to_string());
        }
        let rc = rc.unwrap();
        // mqtt5 properties
        let (props_len, properties_data) =
            Properties::encode_option(self.properties.as_ref(), protocol_version)?;
        length += properties_data.len() + props_len.len();
        let mut writer = MqttWriter::new(length);
        writer.write_u8(FixedHeader::for_type(PacketType::Connack).encode());
        // length
        writer.write_variable_num(length as u32)?;
        writer.write_u8(if self.session_present { 0x01 } else { 0x0 });
        writer.write_u8(rc);
        writer.write_sized(&properties_data, &props_len)?;
        // return true
        Ok(writer.into_vec())
    }

    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        _: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<ConnackPacket> {
        let flags = reader.read_u8()?;
        if flags > 1 {
            return Err("Invalid connack flags, bits 7-1 must be set to 0".to_string());
        }
        let mut packet = ConnackPacket {
            // fixed,
            // length,
            session_present: flags == 1,
            ..ConnackPacket::default()
        };

        if protocol_version == 5 {
            packet.reason_code = if length >= 2 {
                Some(reader.read_u8()?)
            } else {
                Some(0)
            };
        } else {
            if length < 2 {
                return Err("Packet too short".to_string());
            }
            packet.return_code = Some(reader.read_u8()?);
        }
        // mqtt 5 properties
        if protocol_version == 5 && reader.has_more() {
            packet.properties = match reader.read_properties()? {
                None => None,
                Some(props) => Some(ConnackProperties::from_properties(props)?),
            };
        }
        Ok(packet)
    }
}
