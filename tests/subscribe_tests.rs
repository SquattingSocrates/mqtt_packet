mod tests {
    use mqtt_packet_3_5::byte_reader::*;
    use mqtt_packet_3_5::packet::*;
    use mqtt_packet_3_5::structure::*;
    use std::io::{BufReader, Cursor};

    fn dec_from_buf(v: Vec<u8>) -> PacketDecoder<Cursor<Vec<u8>>> {
        let c = Cursor::new(v);
        PacketDecoder::new(ByteReader::new(BufReader::new(c)))
    }

    fn test_decode(name: &str, packet: MqttPacket, buf: Vec<u8>, protocol_version: u8) {
        let mut decoder = dec_from_buf(buf.clone());
        println!("Failed: {}", name);
        assert_eq!(packet, decoder.decode_packet(protocol_version).unwrap());
        println!("Failed encode {}", name);
        assert_eq!(buf, packet.encode(protocol_version).unwrap());
    }

    fn test_decode_error(msg: &str, buf: Vec<u8>, protocol_version: u8) {
        let mut decoder = dec_from_buf(buf.clone());
        println!("Failed: {}", msg);
        assert_eq!(
            Err(msg.to_string()),
            decoder.decode_packet(protocol_version)
        );
    }

    #[test]
    fn test_sub_error_0() {
        test_decode_error(
            "Invalid header flag bits, must be 0x2 for Subscribe packet",
            vec![
                128, 9, // Header (subscribeqos=0length=9)
                0, 6, // Message ID (6)
                0, 4, // Topic length,
                116, 101, 115, 116, // Topic (test)
                0,   // Qos (0)
            ],
            3,
        );
    }

    #[test]
    fn test_subscribe_0() {
        test_decode(
            "subscribe to one topic",
            MqttPacket::Subscribe(SubscribePacket {
                qos: 1,
                message_id: 6,
                subscriptions: vec![Subscription {
                    qos: QoS::QoS0,
                    topic: "test".to_string(),
                    nl: false,
                    rap: false,
                    rh: None,
                }],
                properties: None,
            }),
            vec![
                130, 9, // Header (subscribeqos=1length=9)
                0, 6, // Message ID (6)
                0, 4, // Topic length,
                116, 101, 115, 116, // Topic (test)
                0,   // Qos (0)
            ],
            3,
        );
    }

    #[test]
    fn test_sub_error_1() {
        test_decode_error(
            "Invalid QoS, must be <= 2",
            vec![
                130, 9, // Header (subscribeqos=0length=9)
                0, 6, // Message ID (6)
                0, 4, // Topic length,
                116, 101, 115, 116, // Topic (test)
                3,   // Qos
            ],
            3,
        );
    }

    #[test]
    fn test_sub_error_2() {
        test_decode_error(
            "Invalid subscribe topic flag bits, bits 7-6 must be 0",
            vec![
                130, 10, // Header (subscribeqos=0length=9)
                0, 6, // Message ID (6)
                0, // Property length (0)
                0, 4, // Topic length,
                116, 101, 115, 116,  // Topic (test)
                0x80, // Flags
            ],
            5,
        );
    }

    #[test]
    fn test_sub_error_3() {
        test_decode_error(
            "Invalid retain handling, must be <= 2",
            vec![
                130, 10, // Header (subscribeqos=0length=9)
                0, 6, // Message ID (6)
                0, // Property length (0)
                0, 4, // Topic length,
                116, 101, 115, 116,  // Topic (test)
                0x30, // Flags
            ],
            5,
        );
    }

    #[test]
    fn test_sub_error_4() {
        test_decode_error(
            "Invalid subscribe topic flag bits, bits 7-2 must be 0",
            vec![
                130, 9, // Header (subscribeqos=0length=9)
                0, 6, // Message ID (6)
                0, 4, // Topic length,
                116, 101, 115, 116,  // Topic (test)
                0x08, // Flags
            ],
            3,
        );
    }

    #[test]
    fn test_subscribe_1() {
        test_decode(
            "subscribe to one topic by MQTT 5",
            MqttPacket::Subscribe(SubscribePacket {
                qos: 1,
                message_id: 6,
                subscriptions: vec![Subscription {
                    topic: "test".to_string(),
                    qos: QoS::QoS0,
                    nl: false,
                    rap: true,
                    rh: Some(1),
                }],
                properties: Some(SubscribeProperties {
                    subscription_identifier: 145,
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                }),
            }),
            vec![
                130, 26, // Header (subscribeqos=1length=9)
                0, 6,  // Message ID (6)
                16, // properties length
                11, 145, 1, // subscriptionIdentifier
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
                0, 4, // Topic length,
                116, 101, 115, 116, // Topic (test)
                24, // settings(qos: 0, noLocal: false, Retain as Published: true, retain handling: 1)
            ],
            5,
        );
    }

    #[test]
    fn test_subscribe_2() {
        test_decode(
            "subscribe to three topics",
            MqttPacket::Subscribe(SubscribePacket {
                qos: 1,
                message_id: 6,
                subscriptions: vec![
                    Subscription {
                        topic: "test".to_string(),
                        qos: QoS::QoS0,
                        nl: false,
                        rap: false,
                        rh: None,
                    },
                    Subscription {
                        topic: "uest".to_string(),
                        qos: QoS::QoS1,
                        nl: false,
                        rap: false,
                        rh: None,
                    },
                    Subscription {
                        topic: "tfst".to_string(),
                        qos: QoS::QoS2,
                        nl: false,
                        rap: false,
                        rh: None,
                    },
                ],
                properties: None,
            }),
            vec![
                130, 23, // Header (publishqos=1length=9)
                0, 6, // Message ID (6)
                0, 4, // Topic length,
                116, 101, 115, 116, // Topic (test)
                0,   // Qos (0)
                0, 4, // Topic length
                117, 101, 115, 116, // Topic (uest)
                1,   // Qos (1)
                0, 4, // Topic length
                116, 102, 115, 116, // Topic (tfst)
                2,   // Qos (2)
            ],
            3,
        );
    }

    #[test]
    fn test_subscribe_3() {
        test_decode(
            "subscribe to 3 topics by MQTT 5",
            MqttPacket::Subscribe(SubscribePacket {
                qos: 1,
                message_id: 6,
                properties: Some(SubscribeProperties {
                    subscription_identifier: 145,
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                }),
                subscriptions: vec![
                    Subscription {
                        topic: "test".to_string(),
                        qos: QoS::QoS0,
                        nl: false,
                        rap: true,
                        rh: Some(1),
                    },
                    Subscription {
                        topic: "uest".to_string(),
                        qos: QoS::QoS1,
                        nl: false,
                        rap: false,
                        rh: Some(0),
                    },
                    Subscription {
                        topic: "tfst".to_string(),
                        qos: QoS::QoS2,
                        nl: true,
                        rap: false,
                        rh: Some(0),
                    },
                ],
            }),
            vec![
                130, 40, // Header (subscribeqos=1length=9)
                0, 6,  // Message ID (6)
                16, // properties length
                11, 145, 1, // subscriptionIdentifier
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
                0, 4, // Topic length,
                116, 101, 115, 116, // Topic (test)
                24, // settings(qos: 0, noLocal: false, Retain as Published: true, retain handling: 1)
                0, 4, // Topic length
                117, 101, 115, 116, // Topic (uest)
                1,   // Qos (1)
                0, 4, // Topic length
                116, 102, 115, 116, // Topic (tfst)
                6,   // Qos (2), No Local: true
            ],
            5,
        );
    }

    #[test]
    fn test_suback_0() {
        test_decode(
            "suback",
            MqttPacket::Suback(SubackPacket::new_v3(
                6,
                vec![
                    Granted::QoS0,
                    Granted::QoS1,
                    Granted::QoS2,
                    Granted::Failure,
                ],
            )),
            vec![
                144, 6, // Header
                0, 6, // Message ID
                0, 1, 2, 0x80, // rejected subscription
            ],
            3,
        );
    }

    #[test]
    fn test_suback_1() {
        test_decode(
            "suback",
            MqttPacket::Suback(SubackPacket {
                reason_code: None,
                properties: None,
                message_id: 6,
                granted_reason_codes: vec![
                    SubscriptionReasonCode::GrantedQoS0,
                    SubscriptionReasonCode::GrantedQoS1,
                    SubscriptionReasonCode::GrantedQoS2,
                    SubscriptionReasonCode::UnspecifiedError,
                ],
                granted: vec![],
            }),
            vec![
                144, 7, // Header
                0, 6, // Message ID
                0, // Property length
                0, 1, 2, 128, // Granted qos (0, 1, 2) and a rejected being 0x80
            ],
            5,
        );
    }

    #[test]
    fn test_error_6() {
        test_decode_error(
            "Invalid Granted, must be <= 2 or 0x80",
            vec![
                144, 6, // Header
                0, 6, // Message ID
                0, 1, 2, 0x79, // Granted qos (0, 1, 2) and an invalid code
            ],
            3,
        );
    }

    #[test]
    fn test_suback_2() {
        test_decode(
            "suback MQTT 5",
            MqttPacket::Suback(SubackPacket {
                reason_code: None,
                properties: Some(ConfirmationProperties {
                    reason_string: Some("test".to_string()),
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                }),
                message_id: 6,
                granted_reason_codes: vec![
                    SubscriptionReasonCode::GrantedQoS0,
                    SubscriptionReasonCode::GrantedQoS1,
                    SubscriptionReasonCode::GrantedQoS2,
                    SubscriptionReasonCode::UnspecifiedError,
                ],
                granted: vec![],
            }),
            vec![
                144, 27, // Header
                0, 6,  // Message ID
                20, // properties length
                31, 0, 4, 116, 101, 115, 116, // reasonString
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
                0, 1, 2, 128, // Granted qos (0, 1, 2) and a rejected being 0x80
            ],
            5,
        );
    }
}
