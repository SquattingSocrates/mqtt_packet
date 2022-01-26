use crate::byte_reader::*;
use crate::packet::*;
use crate::structure::*;
use std::io;

impl<R: io::Read> PacketDecoder<R> {
  pub fn decode_unsubscribe_with_length(
    &mut self,
    fixed: FixedHeader,
    length: u32,
    protocol_version: u8,
  ) -> Res<UnsubscribePacket> {
    let mut packet = UnsubscribePacket {
      fixed,
      unsubscriptions: vec![],
      properties: None,
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
