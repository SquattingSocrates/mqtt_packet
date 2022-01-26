use crate::byte_reader::*;
use crate::packet::*;
use crate::structure::*;
use serde::{Deserialize, Serialize};
use std::io;

impl<R: io::Read> PacketDecoder<R> {
    pub fn decode_connect_with_length(
        &mut self,
        fixed: FixedHeader,
        length: u32,
    ) -> Result<ConnectPacket, String> {
        // Parse protocolId
        let protocol_id = self.reader.read_utf8_string()?;
        let protocol_id = match &protocol_id[..] {
            "MQTT" => Protocol::Mqtt,
            "MQIsdp" => Protocol::MQIsdp,
            s => return Err(format!("Invalid protocolId {}", s)),
        };
        // Parse constants version number
        let mut protocol_version = self.reader.read_u8()?;
        if !self.reader.has_more() {
            return Err("Packet too short".to_string());
        }

        if protocol_version >= 128 {
            //   packet.bridgeMode = true
            protocol_version -= 128
        }

        if protocol_version != 3 && protocol_version != 4 && protocol_version != 5 {
            return Err("Invalid protocol version".to_string());
        }

        // if !self.reader.has_more() {
        //     return Err("Packet too short".to_string());
        // }
        let (connect_flags, last_will) = self.decode_connect_flags()?;
        // Parse keepalive
        let keep_alive = self.reader.read_u16()?;
        let connect_properties = self.parse_connect_properties(protocol_version == 5)?;
        // Start parsing payload
        // Parse client_id
        let client_id = self.reader.read_utf8_string()?;
        println!("GOT CLIENT ID {}", client_id);
        let last_will = if connect_flags.will && last_will.is_some() {
            let mut will = last_will.unwrap();
            if protocol_version == 5 {
                will.properties = self.parse_will_properties()?;
            }
            // Parse will topic
            will.topic = Some(self.reader.read_utf8_string()?);
            // Parse will payload
            will.payload = Some(self.reader.read_utf8_string()?);
            Some(will)
        } else {
            last_will
        };

        // Parse username
        let mut user_name = None;
        if connect_flags.user_name {
            user_name = Some(self.reader.read_utf8_string()?);
        }

        // Parse password
        let mut password = None;
        if connect_flags.password {
            password = Some(self.reader.read_utf8_string()?);
        }
        // need for right parse auth packet and self set up
        // this.settings = packet
        Ok(ConnectPacket {
            fixed,
            length,
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

    fn decode_connect_flags(&mut self) -> Result<(ConnectFlags, Option<LastWill>), String> {
        let connect_flags = self.reader.read_u8()?;
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
                properties: WillProperties::default(),
                // properties: Option<LastWillProperties>,
            })
        } else {
            None
        };
        Ok((connect_flags, will_flags))
    }

    fn parse_connect_properties(&mut self, should_parse: bool) -> Res<Option<ConnectProperties>> {
        if !should_parse {
            return Ok(None);
        }
        match self.reader.read_properties()? {
            None => Ok(None),
            Some(prop_list) => Ok(Some(ConnectProperties::from_properties(prop_list)?)),
        }
    }

    fn parse_will_properties(&mut self) -> Res<WillProperties> {
        let props = WillProperties::default();
        match self.reader.read_properties()? {
            None => Ok(props),
            Some(props) => WillProperties::from_properties(props),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
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
        println!("GOT CONNECT BYTE {}. {} {}", byte, byte & 0x4, byte & 0x18);
        ConnectFlags {
            user_name: (byte & 0x80) != 0,    // 0x80 = (1 << 7)
            password: (byte & 0x40) != 0,     // 0x40 = (1 << 6)
            will_retain: (byte & 0x20) != 0,  // 0x20 = (1 << 5)
            will_qos: (byte & 0x18) >> 3,     // 0x18 = 24 = ((1 << 4) + (1 << 3)),
            will: (byte & 0x4) != 0,          // 0x4 = 1 << 2
            clean_session: (byte & 0x2) != 0, // 0x2 = 1 << 2
        }
    }
}
