use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};
use std::io;

const MQISDP_BUF: [u8; 6] = [b'M', b'Q', b'I', b's', b'd', b'p'];
const MQTT_BUF: [u8; 4] = [b'M', b'Q', b'T', b'T'];

impl Packet for ConnectPacket {
    /// This
    fn encode(&self, _: u8) -> Res<Vec<u8>> {
        let ConnectPacket {
            properties,
            protocol_id,
            protocol_version,
            password,
            client_id,
            will,
            clean_session,
            keep_alive,
            user_name,
            ..
        } = self;
        let protocol_version = *protocol_version;
        let mut length = 0;

        // add protocol length
        length += 2 + match protocol_id {
            Protocol::Mqtt => 4,
            Protocol::MQIsdp => 6,
        };

        // Must be 3 or 4 or 5
        if let 3 | 4 | 5 = protocol_version {
            length += 1;
        } else {
            return Err("Invalid protocol version".to_string());
        }

        // ClientId might be omitted in 3.1.1 and 5, but only if cleanSession is set to 1
        if (client_id.is_empty() && protocol_version >= 4 && *clean_session)
            || !client_id.is_empty()
        {
            length += client_id.len() + 2;
        } else {
            if protocol_version < 4 {
                return Err("client_id must be supplied before 3.1.1".to_string());
            }
            if !clean_session {
                return Err("client_id must be given if clean_session set to false".to_string());
            }
        }

        // "keep_alive" Must be a two byte number
        // also add connect flags
        length += 2 + 1;

        // mqtt5 properties
        let (props_len, properties_data) =
            Properties::encode_option(properties.as_ref(), protocol_version)?;
        length += properties_data.len() + props_len.len();

        // If will exists...
        let mut will_retain = false;
        let mut will_qos = None;
        let mut has_will = false;
        let mut will_properties = vec![];
        let mut will_props_len = vec![];
        let mut will_topic: &str = "";
        let mut will_payload: &str = "";
        if let Some(will) = will {
            let LastWill {
                topic,
                payload,
                properties,
                qos,
                retain,
            } = will;
            has_will = true;
            will_retain = *retain;
            will_qos = Some(qos);
            // It must have non-empty topic
            // add topic length if any
            if let Some(t) = topic.as_ref() {
                if t.is_empty() {
                    return Err("Not allowed to use empty will topic".to_string());
                }
                will_topic = t;
                length += t.len() + 2;
            }
            // Payload
            length += 2; // payload length
            if let Some(data) = payload.as_ref() {
                will_payload = data;
                length += data.len();
            }
            // will properties
            if protocol_version == 5 {
                let (l, w) = Properties::encode_option(properties.as_ref(), protocol_version)?;
                will_properties = w;
                will_props_len = l;
                length += will_properties.len() + will_props_len.len();
            }
        }

        // Username
        let mut has_username = false;
        if let Some(user_name) = &user_name {
            has_username = true;
            length += user_name.len() + 2;
        }

        // Password
        let mut has_password = false;
        if let Some(pass) = &password {
            if !has_username {
                return Err("Username is required to use password".to_string());
            }
            has_password = true;
            length += pass.len() + 2;
        }

        let mut writer = MqttWriter::new(length);
        // write header
        writer.write_u8(FixedHeader::for_type(PacketType::Connect).encode());
        // length
        writer.write_variable_num(length as u32)?;
        // protocol id and protocol version
        let proto_vec = match protocol_id {
            Protocol::MQIsdp => MQISDP_BUF.to_vec(),
            Protocol::Mqtt => MQTT_BUF.to_vec(),
        };
        writer.write_u16(proto_vec.len() as u16);
        writer.write_vec(proto_vec);
        writer.write_u8(protocol_version);
        // write connect flags
        writer.write_u8(
            ((has_username as u8) * 0x80) //user_name:  0x80 = (1 << 7)
            | ((has_password as u8) * 0x40) //password:  0x40 = (1 << 6)
            | ((will_retain as u8) * 0x20)  //will_retain:  0x20 = (1 << 5)
            | ((*will_qos.unwrap_or(&0) << 3) & 0x18)     //will_qos:  0x18 = 24 = ((1 << 4) + (1 << 3)),
            | ((has_will as u8) * 0x4) //will:  0x4 = 1 << 2
            | ((*clean_session as u8) * 0x2), //clean_session:  0x2 = 1 << 2)
        );
        // write keep alive
        writer.write_u16(*keep_alive);

        writer.write_sized(&properties_data, &props_len)?;
        // client id
        writer.write_utf8_str(client_id);
        // will properties
        if protocol_version == 5 {
            writer.write_sized(&will_properties, &will_props_len)?;
        }
        // will topic and payload
        if has_will {
            writer.write_utf8_str(will_topic);
            writer.write_utf8_str(will_payload);
        }

        // username
        if let Some(u) = user_name {
            writer.write_utf8_str(u);
        }
        // password
        if let Some(p) = password {
            writer.write_utf8_str(p);
        }
        Ok(writer.into_vec())
    }

    /// Decode connect packet
    fn decode<R: io::Read>(reader: &mut ByteReader<R>, _: FixedHeader, _: u32, _: u8) -> Res<Self> {
        // Parse protocolId
        let protocol_id = reader.read_utf8_string()?;
        let protocol_id = Protocol::from_source(&protocol_id)?;
        // Parse constants version number
        let mut protocol_version = reader.read_u8()?;
        if !reader.has_more() {
            return Err("Packet too short".to_string());
        }

        if protocol_version >= 128 {
            //   packet.bridgeMode = true
            protocol_version -= 128
        }

        if protocol_version != 3 && protocol_version != 4 && protocol_version != 5 {
            return Err("Invalid protocol version".to_string());
        }

        let (connect_flags, last_will) = ConnectFlags::from_byte(reader.read_u8()?)?;
        // Parse keepalive
        let keep_alive = reader.read_u16()?;
        let connect_properties = if protocol_version == 5 {
            match reader.read_properties()? {
                None => None,
                Some(props) => Some(ConnectProperties::from_properties(props)?),
            }
        } else {
            None
        };
        // Start parsing payload
        // Parse client_id
        let client_id = reader.read_utf8_string()?;
        let last_will = if let (Some(mut will), true) = (last_will, connect_flags.will) {
            if protocol_version == 5 {
                will.properties = match reader.read_properties()? {
                    None => None,
                    Some(props) => Some(WillProperties::from_properties(props)?),
                };
            }
            // Parse will topic
            will.topic = Some(reader.read_utf8_string()?);
            // Parse will payload
            will.payload = Some(reader.read_utf8_string()?);
            Some(will)
        } else {
            // since connect_flags.will = false, we don't really care about the last will
            None
        };

        // Parse username
        let mut user_name = None;
        if connect_flags.user_name {
            user_name = Some(reader.read_utf8_string()?);
        }

        // Parse password
        let mut password = None;
        if connect_flags.password {
            password = Some(reader.read_utf8_string()?);
        }
        // need for right parse auth packet and self set up
        Ok(ConnectPacket {
            client_id,
            protocol_version,
            protocol_id,
            clean_session: connect_flags.clean_session,
            keep_alive,
            properties: connect_properties,
            user_name,
            password,
            will: last_will,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ConnectFlags {
    pub user_name: bool,
    pub password: bool,
    pub will_retain: bool,
    pub will_qos: u8,
    pub will: bool,
    pub clean_session: bool,
}

impl ConnectFlags {
    pub fn new(byte: u8) -> ConnectFlags {
        ConnectFlags {
            user_name: (byte & 0x80) != 0,    // 0x80 = (1 << 7)
            password: (byte & 0x40) != 0,     // 0x40 = (1 << 6)
            will_retain: (byte & 0x20) != 0,  // 0x20 = (1 << 5)
            will_qos: (byte & 0x18) >> 3,     // 0x18 = 24 = ((1 << 4) + (1 << 3)),
            will: (byte & 0x4) != 0,          // 0x4 = 1 << 2
            clean_session: (byte & 0x2) != 0, // 0x2 = 1 << 2
        }
    }

    pub fn from_byte(connect_flags: u8) -> Result<(ConnectFlags, Option<LastWill>), String> {
        if connect_flags & 0x1 == 1 {
            // The Server MUST validate that the reserved flag in the CONNECT Control Packet is set to zero and disconnect the Client if it is not zero [MQTT-3.1.2-3]
            return Err("Connect flag bit 0 must be 0, but got 1".to_string());
        }
        let connect_flags = ConnectFlags::new(connect_flags);

        if !connect_flags.will {
            if connect_flags.will_retain {
                return Err(
                    "Will Retain Flag must be set to zero when Will Flag is set to 0".to_string(),
                );
            }
            if connect_flags.will_qos != 0 {
                return Err("Will QoS must be set to zero when Will Flag is set to 0".to_string());
            }
        }

        let will_flags = if connect_flags.will {
            Some(LastWill {
                topic: None,
                payload: None,
                qos: connect_flags.will_qos,
                retain: connect_flags.will_retain,
                properties: None,
            })
        } else {
            None
        };
        Ok((connect_flags, will_flags))
    }
}
