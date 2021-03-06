use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

impl Packet for DisconnectPacket {
    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        _: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<DisconnectPacket> {
        let mut packet = DisconnectPacket {
            reason_code: None,
            properties: None,
        };
        if protocol_version == 5 {
            // response code
            if length > 0 {
                let reason_code = reader.read_u8()?;
                // validate disconnect code
                let reason_code = DisconnectCode::from_byte(reason_code)?;
                packet.reason_code = Some(reason_code);
            } else {
                packet.reason_code = Some(DisconnectCode::NormalDisconnection);
            }
            // properies mqtt 5
            packet.properties = match reader.read_properties()? {
                None => None,
                Some(props) => Some(DisconnectProperties::from_properties(props)?),
            };
        }

        Ok(packet)
    }

    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
        let mut length = if protocol_version == 5 { 1 } else { 0 };
        // properies mqtt 5
        let (props_len, properties_data) =
            Properties::encode_option(self.properties.as_ref(), protocol_version)?;
        length += properties_data.len() + props_len.len();
        let mut writer = MqttWriter::new(length);
        // Header
        writer.write_header(FixedHeader::for_type(PacketType::Disconnect));
        // Length
        writer.write_variable_num(length as u32)?;
        // reason code in header
        if protocol_version == 5 && self.reason_code.is_some() {
            writer.write_u8(self.reason_code.as_ref().unwrap().to_byte());
        }
        // properies mqtt 5
        writer.write_sized(&properties_data, &props_len)?;
        Ok(writer.into_vec())
    }
}
