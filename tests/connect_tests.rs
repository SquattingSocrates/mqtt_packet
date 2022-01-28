mod tests {
    use mqtt_packet::byte_reader::*;
    use mqtt_packet::packet::*;
    use mqtt_packet::structure::*;
    use std::io::{BufReader, Cursor};

    fn dec_from_buf(v: Vec<u8>) -> PacketDecoder<Cursor<Vec<u8>>> {
        let c = Cursor::new(v);
        PacketDecoder::new(ByteReader::new(BufReader::new(c)))
    }

    fn test_decode(name: &str, packet: ConnectPacket, buf: Vec<u8>) {
        let mut decoder = dec_from_buf(buf.clone());
        println!("Failed: {}", name);
        assert_eq!(
            MqttPacket::Connect(packet.clone()),
            decoder.decode_packet(3).unwrap()
        );
        let encoder = PacketEncoder::new();
        assert_eq!(buf, encoder.encode_connect(packet).unwrap());
    }

    fn test_encode_error(msg: &str, packet: ConnectPacket) {
        println!("Failed: {}", msg);
        let encoder = PacketEncoder::new();
        assert_eq!(Err(msg.to_string()), encoder.encode_connect(packet));
    }

    #[test]
    fn decode_bytes_connect() {
        let expected = ConnectPacket {
            fixed: FixedHeader {
                cmd: PacketType::Connect,
                qos: 0,
                dup: false,
                retain: false,
            },
            length: 18,
            protocol_id: Protocol::MQIsdp,
            protocol_version: 3,
            keep_alive: 30,
            clean_session: false,
            user_name: None,
            password: None,
            will: None,
            client_id: "test".to_string(),
            properties: None,
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
        test_decode("Minimal connect", expected, buf);
    }

    #[test]
    fn test_err_without_client_id() {
        let expected = ConnectPacket {
            fixed: FixedHeader {
                cmd: PacketType::Connect,
                qos: 0,
                dup: false,
                retain: false,
            },
            length: 18,
            protocol_id: Protocol::MQIsdp,
            protocol_version: 3,
            keep_alive: 30,
            clean_session: false,
            user_name: None,
            password: None,
            will: None,
            client_id: "".to_string(),
            properties: None,
        };
        test_encode_error("client_id must be supplied before 3.1.1", expected);
    }

    #[test]
    fn test_connect_0() {
        test_decode(
            "connect MQTT 5",
            ConnectPacket {
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
                    properties: Some(WillProperties {
                        will_delay_interval: 1234,
                        payload_format_indicator: false,
                        message_expiry_interval: Some(4321),
                        content_type: Some("test".to_string()),
                        response_topic: Some("topic".to_string()),
                        correlation_data: vec![1, 2, 3, 4],
                        user_properties: [("test".to_string(), vec!["test".to_string()])]
                            .into_iter()
                            .collect::<UserProperties>(), //{ test: 'test' }
                    }),
                    topic: Some("topic".to_string()),
                    payload: Some(String::from_utf8(vec![4, 3, 2, 1]).unwrap()),
                }),
                clean_session: true,
                keep_alive: 30,
                properties: Some(ConnectProperties {
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
                }),
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
            ConnectPacket {
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
                    properties: Some(WillProperties {
                        will_delay_interval: 1234,
                        payload_format_indicator: false,
                        message_expiry_interval: Some(4321),
                        content_type: Some("test".to_string()),
                        response_topic: Some("topic".to_string()),
                        correlation_data: vec![1, 2, 3, 4],
                        user_properties: user_properties.clone(), //{ test: 'test' }
                    }),
                    topic: Some("topic".to_string()),
                    payload: Some(String::from_utf8(vec![]).unwrap()),
                }),
                clean_session: true,
                keep_alive: 30,
                client_id: String::from("test"),
                properties: Some(ConnectProperties {
                    session_expiry_interval: 1234,
                    receive_maximum: 432,
                    maximum_packet_size: Some(100),
                    topic_alias_maximum: 456,
                    request_response_information: true,
                    request_problem_information: true,
                    user_properties,
                    authentication_method: Some("test".to_string()),
                    authentication_data: Some(String::from_utf8(vec![1, 2, 3, 4]).unwrap()),
                }),
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
            ConnectPacket {
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
                    properties: None,
                    topic: Some("topic".to_string()),
                    payload: Some(String::from_utf8(vec![4, 3, 2, 1]).unwrap()),
                }),
                clean_session: true,
                keep_alive: 30,
                client_id: String::from("test"),
                properties: Some(ConnectProperties {
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
                }),
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
            "no client_id with 3.1.1",
            ConnectPacket {
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
                properties: None,
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

    #[test]
    fn multiple_messages_1() {
        let buf = vec![
            // First, a valid connect packet:
            16, 12, // Header
            0, 4, // Protocol ID length
            77, 81, 84, 84, // Protocol ID
            4,  // Protocol version
            2,  // Connect flags
            0, 30, // Keepalive
            0, 0, // Client ID length
            //
            // Then an invalid subscribe packet:
            128, 9, // Header (subscribeqos=0length=9)
            0, 6, // Message ID (6)
            0, 4, // Topic length,
            116, 101, 115, 116, // Topic (test)
            0,   // Qos (0)
            //
            // And another invalid subscribe packet:
            128, 9, // Header (subscribeqos=0length=9)
            0, 6, // Message ID (6)
            0, 4, // Topic length,
            116, 101, 115, 116, // Topic (test)
            0,   // Qos (0)
            //
            // Finally, a valid disconnect packet:
            224, 0, // Header
            // =======================
            // same buffer, but new connection attempt
            // Connect:
            16, 12, // Header
            0, 4, // Protocol ID length
            77, 81, 84, 84, // Protocol ID
            4,  // Protocol version
            2,  // Connect flags
            0, 30, // Keepalive
            0, 0, // Client ID length
            // Disconnect:
            224, 0, // Header
        ];
        let mut decoder = dec_from_buf(buf);
        let mut messages = vec![];
        while decoder.has_more() {
            let msg = decoder.decode_packet(3);
            println!("DECODING {:?}", msg);
            messages.push(msg);
        }
        assert_eq!(6, messages.len());
        assert_eq!(true, messages[0].is_ok());
        assert!(
            if let MqttPacket::Connect(ConnectPacket { .. }) = messages[0].as_ref().unwrap() {
                true
            } else {
                false
            }
        );
        assert_eq!(true, messages[1].is_err());
        assert_eq!(true, messages[2].is_err());
        assert_eq!(true, messages[3].is_ok());
        assert!(if let MqttPacket::Disconnect(DisconnectPacket {
            reason_code: None,
            length: 0,
            ..
        }) = messages[3].as_ref().unwrap()
        {
            true
        } else {
            false
        });
        assert_eq!(true, messages[4].is_ok());
        assert!(
            if let MqttPacket::Connect(ConnectPacket { .. }) = messages[4].as_ref().unwrap() {
                true
            } else {
                false
            }
        );

        assert_eq!(true, messages[5].is_ok());
        assert!(if let MqttPacket::Disconnect(DisconnectPacket {
            reason_code: None,
            length: 0,
            ..
        }) = messages[5].as_ref().unwrap()
        {
            true
        } else {
            false
        });
    }
}
