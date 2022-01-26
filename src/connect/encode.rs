use crate::packet::*;
use crate::structure::*;

impl PacketEncoder {
    pub fn encode_connect(mut self, packet: ConnectPacket, protocol_version: u8) -> Res<Vec<u8>> {
        // let rc = if protocol_version == 5 {
        //     packet.reason_code
        // } else {
        //     packet.return_code
        // };
        // let mut length = 2; // length of rc and sessionHeader

        // // Check return code
        // if rc.is_none() {
        //     return Err("Invalid return code".to_string());
        // }
        // let rc = rc.unwrap();
        // // mqtt5 properties
        // let mut properties_data = if protocol_version == 5 {
        //     if packet.properties.is_some() {
        //         packet.properties.unwrap().encode()
        //     } else {
        //         vec![0]
        //     }
        // } else {
        //     vec![]
        // };
        // length += properties_data.len();
        // self.buf.push(packet.fixed.encode());
        // // length
        // let mut length = PacketEncoder::encode_variable_num(length as u32);
        // self.buf.append(&mut length);
        // self.buf
        //     .push(if packet.session_present { 0x01 } else { 0x0 });
        // self.buf.push(rc);
        // if properties_data.len() > 0 {
        //     self.buf.append(&mut properties_data)
        // }
        // return true
        Ok(self.buf)
    }
}
