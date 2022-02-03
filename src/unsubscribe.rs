use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

impl Packet for UnsubscribePacket {
  /// This
  fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
    // Check message ID
    let mut length = 2;

    // add length of unsubscriptions
    length += self
      .unsubscriptions
      .iter()
      .fold(0, |acc, unsub| acc + unsub.len() + 2);

    // properies mqtt 5
    let (props_len, properties_data) =
      Properties::encode_option(self.properties.as_ref(), protocol_version)?;
    length += properties_data.len() + props_len.len();
    let mut writer = MqttWriter::new(length);
    // header
    writer.write_header(FixedHeader::for_type(PacketType::Unsubscribe));

    // Length
    writer.write_variable_num(length as u32)?;

    // Message ID
    writer.write_u16(self.message_id);

    // properies mqtt 5
    writer.write_sized(&properties_data, &props_len)?;

    // Unsubs
    for unsub in self.unsubscriptions.iter() {
      writer.write_utf8_str(unsub);
    }
    Ok(writer.into_vec())
  }

  fn decode<R: io::Read>(
    reader: &mut ByteReader<R>,
    fixed: FixedHeader,
    _: u32,
    protocol_version: u8,
  ) -> Res<Self> {
    let message_id = reader.read_u16()?;
    let mut packet = UnsubscribePacket {
      qos: fixed.qos,
      unsubscriptions: vec![],
      properties: None,
      message_id,
    };

    // Properties mqtt 5
    if protocol_version == 5 {
      packet.properties = match reader.read_properties()? {
        None => None,
        Some(props) => Some(UnsubscribeProperties::from_properties(props)?),
      };
    }

    if !reader.has_more() {
      return Err("Malformed unsubscribe, no payload specified".to_string());
    }

    while reader.has_more() {
      // Parse topic
      let topic = reader.read_utf8_string()?;
      // Push topic to unsubscriptions
      packet.unsubscriptions.push(topic);
    }
    Ok(packet)
  }
}
