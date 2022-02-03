use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

// convenience constructors for ConfirmationPacket
impl ConfirmationPacket {
    fn with_message_id(cmd: PacketType, message_id: u16) -> ConfirmationPacket {
        ConfirmationPacket {
            cmd,
            message_id,
            properties: None,
            puback_reason_code: None,
            pubcomp_reason_code: None,
        }
    }

    fn puback_v5_builder(
        cmd: PacketType,
        message_id: u16,
        reason_code: PubackPubrecCode,
        properties: Option<ConfirmationProperties>,
    ) -> ConfirmationPacket {
        ConfirmationPacket {
            cmd,
            message_id,
            properties,
            puback_reason_code: Some(reason_code),
            pubcomp_reason_code: None,
        }
    }

    fn pubcomp_v5_builder(
        cmd: PacketType,
        message_id: u16,
        reason_code: PubcompPubrelCode,
        properties: Option<ConfirmationProperties>,
    ) -> ConfirmationPacket {
        ConfirmationPacket {
            cmd,
            message_id,
            properties,
            puback_reason_code: None,
            pubcomp_reason_code: Some(reason_code),
        }
    }

    /// create a correct v3 PUBACK packet. A v3 PUBACK requires
    /// only a 2 byte message_id
    /// # Example
    ///
    /// ```
    /// use mqtt_packet_3_5::{ConfirmationPacket, PacketType};
    /// let packet = ConfirmationPacket::puback_v3(123);
    /// assert_eq!(packet, ConfirmationPacket {
    ///     cmd: PacketType::Puback,
    ///     message_id: 123,
    ///     properties: None,          // v3 has no properties
    ///     puback_reason_code: None,  // v3 has no reason code
    ///     pubcomp_reason_code: None, // v3 has no reason code
    /// });
    ///
    ///
    /// ```
    pub fn puback_v3(message_id: u16) -> ConfirmationPacket {
        ConfirmationPacket::with_message_id(PacketType::Puback, message_id)
    }

    /// create a correct v5 PUBACK packet. A v5 PUBACK requires
    /// a 2 byte message_id, a reason code and possibly empty properties
    pub fn puback_v5(
        message_id: u16,
        reason_code: PubackPubrecCode,
        properties: Option<ConfirmationProperties>,
    ) -> ConfirmationPacket {
        Self::puback_v5_builder(PacketType::Puback, message_id, reason_code, properties)
    }

    /// create a correct v3 PUBREC packet. A v3 PUBREC requires
    /// only a 2 byte message_id
    /// # Example
    ///
    /// ```
    /// use mqtt_packet_3_5::{ConfirmationPacket, PacketType};
    /// let packet = ConfirmationPacket::pubrec_v3(123);
    /// assert_eq!(packet, ConfirmationPacket {
    ///     cmd: PacketType::Pubrec,
    ///     message_id: 123,
    ///     properties: None,          // v3 has no properties
    ///     puback_reason_code: None,  // v3 has no reason code
    ///     pubcomp_reason_code: None, // v3 has no reason code
    /// });
    ///
    ///
    /// ```
    pub fn pubrec_v3(message_id: u16) -> ConfirmationPacket {
        ConfirmationPacket::with_message_id(PacketType::Pubrec, message_id)
    }

    /// create a correct v5 PUBREC packet. A v5 PUBREC requires
    /// a 2 byte message_id, a reason code and possibly empty properties
    pub fn pubrec_v5(
        message_id: u16,
        reason_code: PubackPubrecCode,
        properties: Option<ConfirmationProperties>,
    ) -> ConfirmationPacket {
        Self::puback_v5_builder(PacketType::Pubrec, message_id, reason_code, properties)
    }

    /// create a correct v3 PUBREL packet. A v3 PUBREL requires
    /// only a 2 byte message_id. Since a PUBREL needs to have QoS 1
    /// set there's no need to provide a constructor for this and is taken
    /// care of while encoding
    /// # Example
    ///
    /// ```
    /// use mqtt_packet_3_5::{ConfirmationPacket, PacketType};
    /// let packet = ConfirmationPacket::pubrel_v3(123);
    /// assert_eq!(packet, ConfirmationPacket {
    ///     cmd: PacketType::Pubrel,
    ///     message_id: 123,
    ///     properties: None,          // v3 has no properties
    ///     puback_reason_code: None,  // v3 has no reason code
    ///     pubcomp_reason_code: None, // v3 has no reason code
    /// });
    ///
    ///
    /// ```
    pub fn pubrel_v3(message_id: u16) -> ConfirmationPacket {
        ConfirmationPacket::with_message_id(PacketType::Pubrel, message_id)
    }

    /// create a correct v5 PUBREL packet. A v5 PUBREL requires
    /// a 2 byte message_id, a reason code and possibly empty properties
    pub fn pubrel_v5(
        message_id: u16,
        reason_code: PubcompPubrelCode,
        properties: Option<ConfirmationProperties>,
    ) -> ConfirmationPacket {
        Self::pubcomp_v5_builder(PacketType::Pubrel, message_id, reason_code, properties)
    }

    /// create a correct v3 PUBCOMP packet. A v3 PUBCOMP requires
    /// only a 2 byte message_id
    /// # Example
    ///
    /// ```
    /// use mqtt_packet_3_5::{ConfirmationPacket, PacketType};
    /// let packet = ConfirmationPacket::pubcomp_v3(123);
    /// assert_eq!(packet, ConfirmationPacket {
    ///     cmd: PacketType::Pubcomp,
    ///     message_id: 123,
    ///     properties: None,          // v3 has no properties
    ///     puback_reason_code: None,  // v3 has no reason code
    ///     pubcomp_reason_code: None, // v3 has no reason code
    /// });
    ///
    ///
    /// ```
    pub fn pubcomp_v3(message_id: u16) -> ConfirmationPacket {
        ConfirmationPacket::with_message_id(PacketType::Pubcomp, message_id)
    }

    /// create a correct v5 PUBREL packet. A v5 PUBREL requires
    /// a 2 byte message_id, a reason code and possibly empty properties
    pub fn pubcomp_v5(
        message_id: u16,
        reason_code: PubcompPubrelCode,
        properties: Option<ConfirmationProperties>,
    ) -> ConfirmationPacket {
        Self::pubcomp_v5_builder(PacketType::Pubcomp, message_id, reason_code, properties)
    }
}

impl Packet for ConfirmationPacket {
    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
        //   const dup = (settings.dup && type === 'pubrel') ? protocol.DUP_MASK : 0
        let ConfirmationPacket {
            cmd,
            properties,
            pubcomp_reason_code,
            puback_reason_code,
            message_id,
            ..
        } = &self;
        // let mut length = if protocol_version == 5 { 3 } else { 2 };
        let mut length = 2;
        // Bits 3,2,1 and 0 of the Fixed Header in the PUBREL packet are reserved
        // and MUST be set to 0,0,1 and 0 respectively. The Server MUST treat
        // any other value as malformed and close the
        // Network Connection [MQTT-3.6.1-1].
        let qos = if *cmd == PacketType::Pubrel { 1 } else { 0 };

        // reason code in header
        let code = if protocol_version == 5 {
            match (puback_reason_code, pubcomp_reason_code, cmd) {
                (Some(_), Some(_), t) => {
                    return Err(format!(
                    "Only puback_reason_code OR pubcomp_reason_code can be set simultaneously {:?}",
                    t
                ))
                }
                (Some(code), None, PacketType::Pubrec | PacketType::Puback) => Some(code.to_byte()),
                (None, Some(code), PacketType::Pubcomp | PacketType::Pubrel) => {
                    Some(code.to_byte())
                }
                (x, y, t) => {
                    return Err(format!(
                        "Invalid combination of confirmation type {:?} and codes {:?} | {:?}",
                        t, x, y
                    ))
                }
            }
        } else {
            None
        };

        // properies mqtt 5
        let (props_len, properties_data) =
            Properties::encode_option(properties.as_ref(), protocol_version)?;
        // The Client or Server sending the PUBREL packet MUST use one of
        // the PUBREL Reason Code values [MQTT-3.6.2-1]. The Reason Code
        // and Property Length can be omitted if the Reason Code is 0x00 (Success)
        // and there are no Properties. In this case the PUBREL has a
        // Remaining Length of 2.
        match (code, properties_data.is_empty()) {
            (Some(0), true) => length = 2,
            _ => length += properties_data.len() + props_len.len(),
        };

        let mut writer = MqttWriter::new(length);
        // Header
        let mut header = FixedHeader::encode(&FixedHeader::for_type(*cmd));
        // if fixed.dup {
        //     header |= 0x08;
        // }
        if qos > 0 {
            header |= qos << 1;
        }
        writer.write_u8(header);

        // Length
        writer.write_variable_num(length as u32)?;

        // Message ID
        writer.write_u16(*message_id);
        // maybe write code
        if let (true, Some(c)) = (length > 2, code) {
            writer.write_u8(c);
        }

        // properies mqtt 5
        if length > 2 {
            writer.write_sized(&properties_data, &props_len)?;
        }
        Ok(writer.into_vec())
    }

    /// Decode different confirmation types. Works for PUBACK, PUBREC, PUBREL and PUBCOMP
    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<ConfirmationPacket> {
        let message_id = reader.read_u16()?;
        let mut packet = ConfirmationPacket {
            cmd: fixed.cmd,
            pubcomp_reason_code: None,
            puback_reason_code: None,
            properties: None,
            message_id,
        };
        if protocol_version == 5 {
            let reason_code = if length > 2 {
                // response code
                reader.read_u8()?
            } else {
                0
            };
            // set correct reason code with either read code or
            // from 0 = Success
            match packet.cmd {
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
                packet.properties = match reader.read_properties()? {
                    None => None,
                    Some(props) => Some(ConfirmationProperties::from_properties(props)?),
                };
            }
        }

        // return true
        Ok(packet)
    }
}
