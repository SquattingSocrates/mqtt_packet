use crate::packet::*;
use crate::structure::*;
use std::io;

impl PacketEncoder {
  pub fn encode_unsubscribe(
    mut self,
    packet: UnsubscribePacket,
    protocol_version: u8,
  ) -> Res<Vec<u8>> {
    // Check message ID
    let mut length = 2;

    // add length of unsubscriptions
    length += packet
      .unsubscriptions
      .iter()
      .fold(0, |acc, unsub| acc + unsub.len() + 2);

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

    // Unsubs
    for unsub in packet.unsubscriptions {
      self.write_utf8_string(unsub);
    }
    Ok(self.buf)
  }
}

impl<R: io::Read> PacketDecoder<R> {
  pub fn decode_unsubscribe_with_length(
    &mut self,
    fixed: FixedHeader,
    length: u32,
    protocol_version: u8,
  ) -> Res<UnsubscribePacket> {
    let message_id = self.reader.read_u16()?;
    let mut packet = UnsubscribePacket {
      fixed,
      unsubscriptions: vec![],
      properties: None,
      message_id,
      length,
    };

    // Properties mqtt 5
    if protocol_version == 5 {
      packet.properties = match self.reader.read_properties()? {
        None => None,
        Some(props) => Some(UnsubscribeProperties::from_properties(props)?),
      };
    }

    if !self.reader.has_more() {
      return Err("Malformed unsubscribe, no payload specified".to_string());
    }

    while self.reader.has_more() {
      // Parse topic
      let topic = self.reader.read_utf8_string()?;
      // Push topic to unsubscriptions
      packet.unsubscriptions.push(topic);
    }
    Ok(packet)
  }
}
