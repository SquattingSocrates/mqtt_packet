use crate::packet::*;
use crate::structure::*;

impl PacketEncoder {
    pub fn encode_disconnect(
        &mut self,
        packet: ConfirmationPacket,
        protocol_version: u8,
    ) -> Res<()> {
        let mut length = if protocol_version == 5 { 1 } else { 0 };
        // properies mqtt 5
        let properties_data = PropertyEncoder::encode(packet.properties, protocol_version)?;
        length += properties_data.len();
        // Header
        self.write_header(packet.fixed);
        // Length
        self.write_variable_num(length as u32);
        // reason code in header
        if protocol_version == 5 && packet.reason_code.is_some() {
            self.write_u8(packet.reason_code.unwrap());
        }
        // properies mqtt 5
        self.write_vec(properties_data);
        Ok(())
    }
}
