use crate::packet::*;
use crate::structure::*;

impl PacketEncoder {
    pub fn encode_auth(&mut self, packet: AuthPacket) -> Res<Vec<u8>> {
        Ok(vec![])
    }
}
