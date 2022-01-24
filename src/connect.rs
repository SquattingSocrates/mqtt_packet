// use super::packet::*;
use crate::byte_reader::*;
use crate::structure::*;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(PartialEq, Debug)]
pub struct LastWill {
    topic: Option<String>,
    payload: Option<String>,
    qos: u8,
    retain: bool,
    properties: WillProperties,
}

#[derive(PartialEq, Debug)]
pub struct ExtendedAuth {
    authentication_method: String,
    authentication_data: String,
}

#[derive(PartialEq, Debug)]
pub struct ConnectProperties {
    // defaults to 0
    pub session_expiry_interval: u32,
    // defaults to 65,535
    pub receive_maximum: u16,
    // if None then no limit
    pub maximum_packet_size: Option<u32>,
    // default value is 0
    pub topic_alias_maximum: u16,
    // default is false
    pub request_response_information: bool,
    // default is true
    pub request_problem_information: bool,
    // default is just an empty hashMap
    pub user_properties: UserProperties,
    // default is None
    // pub extended_auth: Option<ExtendedAuth>,
    pub authentication_method: Option<String>,
    pub authentication_data: Option<String>,
}

impl Default for ConnectProperties {
    fn default() -> ConnectProperties {
        ConnectProperties {
            session_expiry_interval: 0,
            receive_maximum: 0xffff,
            maximum_packet_size: None,
            topic_alias_maximum: 0,
            request_response_information: false,
            request_problem_information: true,
            user_properties: UserProperties::new(),
            // extended_auth: None,
            authentication_method: None,
            authentication_data: None,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct WillProperties {
    /// default value is false, both if set to false
    /// and when no value was provided
    pub payload_format_indicator: bool,
    /// None if no value was given, because
    /// apparently 0 is a valid expiry
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<String>,
    /// 0 is default when no value was provided
    pub will_delay_interval: u32,
    pub user_properties: UserProperties,
}

impl Default for WillProperties {
    fn default() -> WillProperties {
        WillProperties {
            payload_format_indicator: false,
            message_expiry_interval: None,
            content_type: None,
            response_topic: None,
            correlation_data: None,
            will_delay_interval: 0,
            user_properties: UserProperties::new(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct ConnectPacket {
    fixed: FixedHeader,
    length: u32,
    client_id: String,
    protocol_version: u8,
    protocol_id: Protocol,
    clean_session: bool,
    keep_alive: u16,
    user_name: Option<String>,
    password: Option<String>,
    /// a last will is not mandatory
    will: Option<LastWill>,
    properties: ConnectProperties,
}

pub struct ConnectPacketDecoder<R: io::Read> {
    reader: ByteReader<R>,
}

impl<R: io::Read> ConnectPacketDecoder<R> {
    pub fn new(reader: ByteReader<R>) -> ConnectPacketDecoder<R> {
        ConnectPacketDecoder { reader }
    }

    pub fn decode_connect_flags(
        &mut self,
        parse_properties: bool,
    ) -> Result<(ConnectFlags, Option<LastWill>), String> {
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

    pub fn decode_bytes(&mut self) -> Res<ConnectPacket> {
        let (length, fixed) = self.reader.read_header()?;
        self.decode_bytes_with_length(fixed, length)
    }

    pub fn decode_bytes_with_length(
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
        println!("PARSED PROTOCOL NAME {:?}", protocol_id);
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

        println!("PARSED PROTOCOL VERSION {}", protocol_version);
        // if !self.reader.has_more() {
        //     return Err("Packet too short".to_string());
        // }
        let (connect_flags, last_will) = self.decode_connect_flags(protocol_version == 5)?;
        // Parse keepalive
        let keep_alive = self.reader.read_u16()?;
        println!("PARSED FLAGS {:?} {:?}", connect_flags, last_will);
        let connect_properties = self.parse_connect_properties(protocol_version == 5)?;
        println!("PARSED CONNECT PROPS");
        // Start parsing payload
        // Parse client_id
        let client_id = self.reader.read_utf8_string()?;
        println!("PARSED CLIENT ID");
        let last_will = if connect_flags.will && last_will.is_some() {
            let mut will = last_will.unwrap();
            if protocol_version == 5 {
                will.properties = self.parse_will_properties()?;
            }
            // Parse will topic
            will.topic = Some(self.reader.read_utf8_string()?);
            println!("PARSED WILL TOPIC {:?} {}", will.topic, will.qos);
            // Parse will payload
            will.payload = Some(self.reader.read_utf8_string()?);
            Some(will)
        } else {
            last_will
        };
        println!("PARSED LAST WILL");

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

    fn parse_connect_properties(&mut self, should_parse: bool) -> Res<ConnectProperties> {
        let mut props = ConnectProperties::default();
        if !should_parse {
            return Ok(props);
        }
        for p in self.reader.read_properties()? {
            match p {
                (0x11, PropType::U32(v)) => props.session_expiry_interval = v,
                (0x15, PropType::String(v)) => props.authentication_method = Some(v),
                (0x16, PropType::String(v)) => props.authentication_data = Some(v),
                (0x17, PropType::Bool(v)) => props.request_problem_information = v,
                (0x19, PropType::Bool(v)) => props.request_response_information = v,
                (0x21, PropType::U16(v)) => props.receive_maximum = v,
                (0x22, PropType::U16(v)) => props.topic_alias_maximum = v,
                (0x26, PropType::Map(v)) => props.user_properties = v,
                (0x27, PropType::U32(v)) => props.maximum_packet_size = Some(v),
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(props)
    }

    fn parse_will_properties(&mut self) -> Res<WillProperties> {
        let mut props = WillProperties::default();
        for p in self.reader.read_properties()? {
            match p {
                (0x01, PropType::Bool(v)) => props.payload_format_indicator = v,
                (0x02, PropType::U32(v)) => props.message_expiry_interval = Some(v),
                (0x03, PropType::String(s)) => props.content_type = Some(s),
                (0x08, PropType::String(s)) => props.response_topic = Some(s),
                (0x09, PropType::String(s)) => props.correlation_data = Some(s),
                (0x18, PropType::U32(v)) => props.will_delay_interval = v,
                (0x26, PropType::Map(v)) => props.user_properties = v,
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(props)
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

mod tests {
    use crate::connect::*;
    use std::io::{BufReader, Cursor};

    fn dec_from_buf(v: Vec<u8>) -> ConnectPacketDecoder<Cursor<Vec<u8>>> {
        let c = Cursor::new(v);
        ConnectPacketDecoder::new(ByteReader::new(BufReader::new(c)))
    }

    fn test_decode(name: &str, packet: &ConnectPacket, buf: Vec<u8>) {
        let mut decoder = dec_from_buf(buf);
        println!("Failed: {}", name);
        assert_eq!(*packet, decoder.decode_bytes().unwrap());
    }

    fn test_encode(name: &str, packet: ConnectPacket, buf: Vec<u8>) {
        println!("NOT IMPLEMENTED");
    }

    #[test]
    fn decode_bytes_connect() {
        let mut expected = ConnectPacket {
            fixed: FixedHeader {
                cmd: PacketType::Connect,
                qos: 0,
                dup: false,
                retain: false,
            },
            length: 18,
            protocol_id: Protocol::from_str("MQIsdp"),
            protocol_version: 3,
            keep_alive: 30,
            clean_session: false,
            user_name: None,
            password: None,
            will: None,
            client_id: "test".to_string(),
            properties: ConnectProperties::default(),
        };
        let buf = vec![
            16, 18, // Header
            0, 6, // Protocol ID length
            77, 81, 73, 115, 100, 112, // Protocol ID
            3,   // Protocol version
            0,   // Connect flags
            0, 30, // Keepalive
            0, 4, // Client ID length
            116, 101, 115, 116, // Client ID
        ];
        test_decode("Minimal connect", &expected, buf);
        // test no client_id
        expected.client_id = String::new();
        let buf = vec![
            16, 18, // Header
            0, 6, // Protocol ID length
            77, 81, 73, 115, 100, 112, // Protocol ID
            3,   // Protocol version
            0,   // Connect flags
            0, 30, // Keepalive
            0, 0, // Client ID length
        ];
        test_decode("Minimal connect", &expected, buf);
    }

    #[test]
    fn test_connect_0() {
        test_decode(
            "connect MQTT 5",
            &ConnectPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Connect,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 125,
                protocol_id: Protocol::Mqtt,
                protocol_version: 5,
                user_name: None,
                password: None,
                will: Some(LastWill {
                    retain: true,
                    qos: 2,
                    properties: WillProperties {
                        will_delay_interval: 1234,
                        payload_format_indicator: false,
                        message_expiry_interval: Some(4321),
                        content_type: Some("test".to_string()),
                        response_topic: Some("topic".to_string()),
                        correlation_data: Some(String::from_utf8(vec![1, 2, 3, 4]).unwrap()),
                        user_properties: [("test".to_string(), vec!["test".to_string()])]
                            .into_iter()
                            .collect::<UserProperties>(), //{ test: 'test' }
                    },
                    topic: Some("topic".to_string()),
                    payload: Some(String::from_utf8(vec![4, 3, 2, 1]).unwrap()),
                }),
                clean_session: true,
                keep_alive: 30,
                properties: ConnectProperties {
                    session_expiry_interval: 1234,
                    receive_maximum: 432,
                    maximum_packet_size: Some(100),
                    topic_alias_maximum: 456,
                    request_response_information: true,
                    request_problem_information: true,
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(), // { test: 'test' },
                    authentication_method: Some("test".to_string()),
                    authentication_data: Some(String::from_utf8(vec![1, 2, 3, 4]).unwrap()),
                },
                client_id: "test".to_string(),
            },
            vec![
                16, 125, // Header
                0, 4, // Protocol ID length
                77, 81, 84, 84, // Protocol ID
                5,  // Protocol version
                54, // Connect flags
                0, 30, // Keepalive
                47, // properties length
                17, 0, 0, 4, 210, // sessionExpiryInterval
                33, 1, 176, // receiveMaximum
                39, 0, 0, 0, 100, // maximumPacketSize
                34, 1, 200, // topicAliasMaximum
                25, 1, // requestResponseInformation
                23, 1, // requestProblemInformation,
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties,
                21, 0, 4, 116, 101, 115, 116, // authenticationMethod
                22, 0, 4, 1, 2, 3, 4, // authenticationData
                0, 4, // Client ID length
                116, 101, 115, 116, // Client ID
                47,  // will properties
                24, 0, 0, 4, 210, // will delay interval
                1, 0, // payload format indicator
                2, 0, 0, 16, 225, // message expiry interval
                3, 0, 4, 116, 101, 115, 116, // content type
                8, 0, 5, 116, 111, 112, 105, 99, // response topic
                9, 0, 4, 1, 2, 3, 4, // corelation data
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // user properties
                0, 5, // Will topic length
                116, 111, 112, 105, 99, // Will topic
                0, 4, // Will payload length
                4, 3, 2, 1, // Will payload
            ],
        );
    }

    #[test]
    fn test_connect_1() {
        let user_properties = [("test".to_string(), vec!["test".to_string()])]
            .into_iter()
            .collect::<UserProperties>();
        test_decode(
            "connect MQTT 5 with will properties but with empty will payload",
            &ConnectPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Connect,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 121,
                protocol_id: Protocol::Mqtt,
                protocol_version: 5,
                user_name: None,
                password: None,
                will: Some(LastWill {
                    retain: true,
                    qos: 2,
                    properties: WillProperties {
                        will_delay_interval: 1234,
                        payload_format_indicator: false,
                        message_expiry_interval: Some(4321),
                        content_type: Some("test".to_string()),
                        response_topic: Some("topic".to_string()),
                        correlation_data: Some(String::from_utf8(vec![1, 2, 3, 4]).unwrap()),
                        user_properties: user_properties.clone(), //{ test: 'test' }
                    },
                    topic: Some("topic".to_string()),
                    payload: Some(String::from_utf8(vec![]).unwrap()),
                }),
                clean_session: true,
                keep_alive: 30,
                client_id: String::from("test"),
                properties: ConnectProperties {
                    session_expiry_interval: 1234,
                    receive_maximum: 432,
                    maximum_packet_size: Some(100),
                    topic_alias_maximum: 456,
                    request_response_information: true,
                    request_problem_information: true,
                    user_properties,
                    authentication_method: Some("test".to_string()),
                    authentication_data: Some(String::from_utf8(vec![1, 2, 3, 4]).unwrap()),
                },
            },
            vec![
                16, 121, // Header
                0, 4, // Protocol ID length
                77, 81, 84, 84, // Protocol ID
                5,  // Protocol version
                54, // Connect flags
                0, 30, // Keepalive
                47, // properties length
                17, 0, 0, 4, 210, // sessionExpiryInterval
                33, 1, 176, // receiveMaximum
                39, 0, 0, 0, 100, // maximumPacketSize
                34, 1, 200, // topicAliasMaximum
                25, 1, // requestResponseInformation
                23, 1, // requestProblemInformation,
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties,
                21, 0, 4, 116, 101, 115, 116, // authenticationMethod
                22, 0, 4, 1, 2, 3, 4, // authenticationData
                0, 4, // Client ID length
                116, 101, 115, 116, // Client ID
                47,  // will properties
                24, 0, 0, 4, 210, // will delay interval
                1, 0, // payload format indicator
                2, 0, 0, 16, 225, // message expiry interval
                3, 0, 4, 116, 101, 115, 116, // content type
                8, 0, 5, 116, 111, 112, 105, 99, // response topic
                9, 0, 4, 1, 2, 3, 4, // corelation data
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // user properties
                0, 5, // Will topic length
                116, 111, 112, 105, 99, // Will topic
                0, 0, // Will payload length
            ],
        );
    }

    #[test]
    fn test_connect_2() {
        test_decode(
            "connect MQTT 5 w/o will properties",
            &ConnectPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Connect,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 78,
                protocol_id: Protocol::Mqtt,
                protocol_version: 5,
                user_name: None,
                password: None,
                will: Some(LastWill {
                    retain: true,
                    qos: 2,
                    properties: WillProperties::default(),
                    topic: Some("topic".to_string()),
                    payload: Some(String::from_utf8(vec![4, 3, 2, 1]).unwrap()),
                }),
                clean_session: true,
                keep_alive: 30,
                client_id: String::from("test"),
                properties: ConnectProperties {
                    session_expiry_interval: 1234,
                    receive_maximum: 432,
                    maximum_packet_size: Some(100),
                    topic_alias_maximum: 456,
                    request_response_information: true,
                    request_problem_information: true,
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                    authentication_method: Some("test".to_string()),
                    authentication_data: Some(String::from_utf8(vec![1, 2, 3, 4]).unwrap()),
                },
            },
            vec![
                16, 78, // Header
                0, 4, // Protocol ID length
                77, 81, 84, 84, // Protocol ID
                5,  // Protocol version
                54, // Connect flags
                0, 30, // Keepalive
                47, // properties length
                17, 0, 0, 4, 210, // sessionExpiryInterval
                33, 1, 176, // receiveMaximum
                39, 0, 0, 0, 100, // maximumPacketSize
                34, 1, 200, // topicAliasMaximum
                25, 1, // requestResponseInformation
                23, 1, // requestProblemInformation,
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties,
                21, 0, 4, 116, 101, 115, 116, // authenticationMethod
                22, 0, 4, 1, 2, 3, 4, // authenticationData
                0, 4, // Client ID length
                116, 101, 115, 116, // Client ID
                0,   // will properties
                0, 5, // Will topic length
                116, 111, 112, 105, 99, // Will topic
                0, 4, // Will payload length
                4, 3, 2, 1, // Will payload
            ],
        );
    }

    #[test]
    fn test_connect_3() {
        test_decode(
            "no clientId with 3.1.1",
            &ConnectPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Connect,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 12,
                protocol_id: Protocol::Mqtt,
                protocol_version: 4,
                user_name: None,
                password: None,
                will: None,
                clean_session: true,
                keep_alive: 30,
                client_id: String::new(),
                properties: ConnectProperties::default(),
            },
            vec![
                16, 12, // Header
                0, 4, // Protocol ID length
                77, 81, 84, 84, // Protocol ID
                4,  // Protocol version
                2,  // Connect flags
                0, 30, // Keepalive
                0, 0, // Client ID length
            ],
        );
    }
}
