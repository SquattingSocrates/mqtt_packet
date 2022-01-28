use crate::packet::*;
use crate::structure::*;
use std::io;

impl PacketEncoder {
    pub fn encode_confirmation(
        mut self,
        packet: ConfirmationPacket,
        protocol_version: u8,
    ) -> Res<Vec<u8>> {
        //   const dup = (settings.dup && type === 'pubrel') ? protocol.DUP_MASK : 0
        let ConfirmationPacket {
            fixed,
            properties,
            pubcomp_reason_code,
            puback_reason_code,
            message_id,
            ..
        } = packet;
        let mut length = if protocol_version == 5 { 3 } else { 2 };
        let qos = if fixed.cmd == PacketType::Pubrel {
            1
        } else {
            0
        };

        // properies mqtt 5
        let mut properties_data = PropertyEncoder::encode(properties, protocol_version)?;
        if properties_data[..] == [0] {
            properties_data = vec![];
        }
        length += properties_data.len();

        // Header
        let mut header = FixedHeader::encode(&fixed);
        if fixed.dup {
            header |= 0x08;
        }
        if qos > 0 {
            header |= qos << 1;
        }
        self.buf.push(header);

        // Length
        self.write_variable_num(length as u32)?;

        // Message ID
        self.write_u16(message_id);

        // reason code in header
        if protocol_version == 5 {
            let code = match (puback_reason_code, pubcomp_reason_code, fixed.cmd) {
                (Some(_), Some(_), t) => {
                    return Err(format!(
                    "Only puback_reason_code OR pubcomp_reason_code can be set simultaneously {:?}",
                    t
                ))
                }
                (Some(code), None, PacketType::Pubrec | PacketType::Puback) => code.to_byte(),
                (None, Some(code), PacketType::Pubcomp | PacketType::Pubrel) => code.to_byte(),
                (x, y, t) => {
                    return Err(format!(
                        "Invalid combination of confirmation type {:?} and codes {:?} | {:?}",
                        t, x, y
                    ))
                }
            };
            self.buf.push(code);
        }

        // properies mqtt 5
        self.write_vec(properties_data);
        Ok(self.buf)
    }
}

impl<R: io::Read> PacketDecoder<R> {
    /// Decode different confirmation types. Works for PUBACK, PUBREC, PUBREL and PUBCOMP
    pub fn decode_confirmation_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<ConfirmationPacket> {
        let message_id = self.reader.read_u16()?;
        let mut packet = ConfirmationPacket {
            fixed,
            pubcomp_reason_code: None,
            puback_reason_code: None,
            properties: None,
            message_id,
            length,
        };
        if protocol_version == 5 {
            let reason_code = if length > 2 {
                // response code
                self.reader.read_u8()?
            } else {
                0
            };
            // set correct reason code with either read code or
            // from 0 = Success
            match packet.fixed.cmd {
                PacketType::Puback | PacketType::Pubrec => {
                    packet.puback_reason_code = Some(PubackPubrecCode::from_byte(reason_code)?);
                }
                PacketType::Pubrel | PacketType::Pubcomp => {
                    packet.pubcomp_reason_code = Some(PubcompPubrelCode::from_byte(reason_code)?);
                }
                t => {
                    return Err(format!(
                        "Something went horribly wrong. Trying to decode confirmation from {:?}",
                        t
                    ))
                }
            }

            if length > 3 {
                // properies mqtt 5
                packet.properties = match self.reader.read_properties()? {
                    None => None,
                    Some(props) => Some(ConfirmationProperties::from_properties(props)?),
                };
            }
        }

        // return true
        Ok(packet)
    }
}
