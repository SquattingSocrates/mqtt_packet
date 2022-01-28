use crate::packet::*;
use crate::structure::*;
use std::io;

impl PacketEncoder {
    pub fn encode_unsuback(mut self, packet: UnsubackPacket, protocol_version: u8) -> Res<Vec<u8>> {
        // Check message ID
        let mut length = 2;

        // add length of unsubscriptions
        length += packet.granted.len();

        // properies mqtt 5
        println!("PROPERTIES  {:?}", packet.properties);
        let properties_data = PropertyEncoder::encode(packet.properties, protocol_version)?;
        println!("PROPERTIES DATA {:?}", properties_data);
        length += properties_data.len();

        // header
        self.write_header(packet.fixed);

        // Length
        self.write_variable_num(length as u32)?;

        // Message ID
        self.write_u16(packet.message_id);

        // properies mqtt 5
        self.write_vec(properties_data);

        // Granted
        for g in packet.granted {
            self.write_u8(g.to_byte());
        }
        Ok(self.buf)
    }
}

impl<R: io::Read> PacketDecoder<R> {
    pub fn decode_unsuback_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<UnsubackPacket> {
        let message_id = self.reader.read_u16()?;

        if (protocol_version == 3 || protocol_version == 4) && length != 2 {
            return Err("Malformed unsuback, payload length must be 2".to_string());
        }
        if length == 0 {
            return Err("Malformed unsuback, no payload specified".to_string());
        }
        let mut packet = UnsubackPacket {
            fixed,
            properties: None,
            granted: vec![],
            message_id,
            length,
        };

        // Properties mqtt 5
        if protocol_version == 5 {
            packet.properties = match self.reader.read_properties()? {
                None => None,
                Some(props) => Some(ConfirmationProperties::from_properties(props)?),
            };
            // Parse granted QoSes

            while self.reader.has_more() {
                let code = UnsubackCode::from_byte(self.reader.read_u8()?)?;
                packet.granted.push(code);
            }
        }
        Ok(packet)
    }
}
