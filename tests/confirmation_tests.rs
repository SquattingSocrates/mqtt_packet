mod tests {
    use mqtt_packet_3_5::byte_reader::*;
    use mqtt_packet_3_5::packet::*;
    use mqtt_packet_3_5::structure::*;
    use std::io::{BufReader, Cursor};

    fn dec_from_buf(v: Vec<u8>) -> PacketDecoder<Cursor<Vec<u8>>> {
        let c = Cursor::new(v);
        PacketDecoder::new(ByteReader::new(BufReader::new(c)))
    }

    fn test_decode(name: &str, packet: ConfirmationPacket, buf: Vec<u8>, protocol_version: u8) {
        let mut decoder = dec_from_buf(buf.clone());
        println!("Failed: {}", name);
        match packet.fixed.cmd {
            PacketType::Puback => assert_eq!(
                MqttPacket::Puback(packet.clone()),
                decoder.decode_packet(protocol_version).unwrap()
            ),
            PacketType::Pubrel => assert_eq!(
                MqttPacket::Pubrel(packet.clone()),
                decoder.decode_packet(protocol_version).unwrap()
            ),
            PacketType::Pubrec => assert_eq!(
                MqttPacket::Pubrec(packet.clone()),
                decoder.decode_packet(protocol_version).unwrap()
            ),
            PacketType::Pubcomp => assert_eq!(
                MqttPacket::Pubcomp(packet.clone()),
                decoder.decode_packet(protocol_version).unwrap()
            ),
            _ => panic!("Should only use confirmation types"),
        }
    }

    fn test_encode(name: &str, packet: ConfirmationPacket, buf: Vec<u8>) {
        println!("Failed encode {}", name);
        let encoder = PacketEncoder::new();
        assert_eq!(buf, encoder.encode_confirmation(packet, 5).unwrap());
    }

    fn test_decode_error(msg: &str, buf: Vec<u8>) {
        let mut decoder = dec_from_buf(buf.clone());
        println!("Failed: {}", msg);
        assert_eq!(Err(msg.to_string()), decoder.decode_packet(5));
    }

    #[test]
    fn test_puback_0() {
        test_decode(
            "Version 5 PUBACK test 1",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Puback,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 2,
                pubcomp_reason_code: None,
                puback_reason_code: Some(PubackPubrecCode::Success),
                properties: None,
                message_id: 42,
            },
            vec![
                64, 2, // Fixed Header (PUBACK, Remaining Length)
                0,
                42, // Variable Header (2 Bytes: Packet Identifier 42, Implied Reason code: Success, Implied no properties)
            ],
            5,
        );
    }

    #[test]
    fn test_puback_1() {
        let packet = ConfirmationPacket {
            fixed: FixedHeader {
                cmd: PacketType::Puback,
                qos: 0,
                dup: false,
                retain: false,
            },
            length: 3,
            pubcomp_reason_code: None,
            puback_reason_code: Some(PubackPubrecCode::Success),
            properties: None,
            message_id: 42,
        };
        test_decode(
            "Version 5 PUBACK test 2",
            packet.clone(),
            vec![
                64, 3, // Fixed Header (PUBACK, Remaining Length)
                0, 42,
                0, // Variable Header (2 Bytes: Packet Identifier 42, Reason code: 0 Success, Implied no properties)
            ],
            5,
        );
        // encoder should always write reason code
        test_encode(
            "Version 5 PUBACK test 2",
            packet.clone(),
            vec![
                64, 3, // Fixed Header (PUBACK, Remaining Length)
                0, 42,
                0, // Variable Header (2 Bytes: Packet Identifier 42, Reason code: 0 Success, Implied no properties)
            ],
        );
    }
    #[test]
    fn test_puback_2() {
        test_decode(
            "Version 5 PUBACK test 3",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Puback,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 4,
                pubcomp_reason_code: None,
                puback_reason_code: Some(PubackPubrecCode::Success),
                properties: None,
                message_id: 42,
            },
            vec![
                64, 4, // Fixed Header (PUBACK, Remaining Length)
                0, 42,
                0, // Variable Header (2 Bytes: Packet Identifier 42, Reason code: 0 Success)
                0, // no properties
            ],
            5,
        );
    }

    #[test]
    fn test_puback_3() {
        test_decode(
            "puback",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Puback,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 2,
                pubcomp_reason_code: None,
                puback_reason_code: None,
                properties: None,
                message_id: 2,
            },
            vec![
                64, 2, // Header
                0, 2, // Message ID
            ],
            3,
        );
    }

    #[test]
    fn test_confirmation_error_1() {
        test_decode_error(
            // "Invalid header flag bits, must be 0x0 for puback packet",
            "Flags 1 should not be set for type Puback",
            vec![
                65, 2, // Header
                0, 2, // Message ID
            ],
        );
    }

    #[test]
    fn test_puback_4() {
        test_decode(
            "puback with reason and no MQTT 5 properties",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Puback,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 3,
                pubcomp_reason_code: None,
                puback_reason_code: Some(PubackPubrecCode::NoMatchingSubscribers),
                properties: None,
                message_id: 2,
            },
            vec![
                64, 3, // Header
                0, 2,  // Message ID
                16, // reason code
            ],
            5,
        );
    }

    #[test]
    fn test_puback_5() {
        test_decode(
            "puback MQTT 5 properties",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Puback,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 24,
                pubcomp_reason_code: None,
                puback_reason_code: Some(PubackPubrecCode::NoMatchingSubscribers),
                message_id: 2,
                properties: Some(ConfirmationProperties {
                    reason_string: Some("test".to_string()),
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                }),
            },
            vec![
                64, 24, // Header
                0, 2,  // Message ID
                16, // reason code
                20, // properties length
                31, 0, 4, 116, 101, 115, 116, // reasonString
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            ],
            5,
        );
    }

    #[test]
    fn test_confirmation_error_2() {
        test_decode_error(
            "Invalid puback/pubrec code 17",
            vec![
                64, 4, // Header
                0, 2,    // Message ID
                0x11, // reason code
                0,    // properties length
            ],
        );
    }

    #[test]
    fn test_pubrec_1() {
        test_decode(
            "pubrec",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Pubrec,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 2,
                pubcomp_reason_code: None,
                puback_reason_code: None,
                properties: None,
                message_id: 2,
            },
            vec![
                80, 2, // Header
                0, 2, // Message ID
            ],
            4,
        );
    }

    #[test]
    fn test_pubrec_5() {
        test_decode(
            "pubrec",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Pubrec,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 2,
                pubcomp_reason_code: None,
                puback_reason_code: Some(PubackPubrecCode::Success),
                properties: None,
                message_id: 2,
            },
            vec![
                80, 2, // Header
                0, 2, // Message ID
            ],
            5,
        );
    }

    #[test]
    fn test_confirmation_error_6() {
        test_decode_error(
            "Flags 1 should not be set for type Pubrec",
            vec![
                81, 2, // Header
                0, 2, // Message ID
            ],
        );
    }

    #[test]
    fn test_pubrec_7() {
        test_decode(
            "pubrec MQTT 5 properties",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Pubrec,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 24,
                pubcomp_reason_code: None,
                puback_reason_code: Some(PubackPubrecCode::NoMatchingSubscribers),
                message_id: 2,
                properties: Some(ConfirmationProperties {
                    reason_string: Some("test".to_string()),
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                }),
            },
            vec![
                80, 24, // Header
                0, 2,  // Message ID
                16, // reason code
                20, // properties length
                31, 0, 4, 116, 101, 115, 116, // reasonString
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            ],
            5,
        );
    }

    #[test]
    fn test_pubrel_8() {
        test_decode(
            "pubrel",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Pubrel,
                    qos: 1,
                    dup: false,
                    retain: false,
                },
                length: 2,
                pubcomp_reason_code: None,
                puback_reason_code: None,
                properties: None,
                message_id: 2,
            },
            vec![
                98, 2, // Header
                0, 2, // Message ID
            ],
            3,
        );
    }

    #[test]
    fn test_confirmation_error_9() {
        test_decode_error(
            "Invalid pubcomp/pubrel code 17",
            vec![
                98, 4, // Header
                0, 2,    // Message ID
                0x11, // Reason code
                0,    // Properties length
            ],
        );
    }

    #[test]
    // Where a flag bit is marked as “Reserved” in Table 2.2 - Flag Bits, it is reserved for future use and MUST be set to the value listed in that table [MQTT-2.2.2-1]. If invalid flags are received, the receiver MUST close the Network Connection [MQTT-2.2.2-2]
    fn test_confirmation_error_10() {
        test_decode_error(
            "Invalid header flag bits, must be 0x2 for Pubrel packet",
            vec![
                96, 2, // Header
                0, 2, // Message ID
            ],
        );
    }

    #[test]
    fn test_pubrel_11() {
        test_decode(
            "pubrel MQTT5 properties",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Pubrel,
                    qos: 1,
                    dup: false,
                    retain: false,
                },
                length: 24,
                pubcomp_reason_code: Some(PubcompPubrelCode::PacketIdentifierNotFound),
                puback_reason_code: None,
                message_id: 2,
                properties: Some(ConfirmationProperties {
                    reason_string: Some("test".to_string()),
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                }),
            },
            vec![
                98, 24, // Header
                0, 2,    // Message ID
                0x92, // reason code
                20,   // properties length
                31, 0, 4, 116, 101, 115, 116, // reasonString
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            ],
            5,
        );
    }

    #[test]
    fn test_confirmation_error_12() {
        test_decode_error(
            "Invalid pubcomp/pubrel code 16",
            vec![
                98, 4, // Header
                0, 2,  // Message ID
                16, // reason code
                0,  // properties length
            ],
        );
    }

    #[test]
    fn test_pubcomp_13() {
        test_decode(
            "pubcomp",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Pubcomp,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 2,
                pubcomp_reason_code: Some(PubcompPubrelCode::Success),
                puback_reason_code: None,
                message_id: 2,
                properties: None,
            },
            vec![
                112, 2, // Header
                0, 2, // Message ID
            ],
            5,
        );
    }

    #[test]
    fn test_confirmation_error_14() {
        test_decode_error(
            "Flags 1 should not be set for type Pubcomp",
            vec![
                113, 2, // Header
                0, 2, // Message ID
            ],
        );
    }

    #[test]
    fn test_pubcomp_15() {
        test_decode(
            "pubcomp MQTT 5 properties",
            ConfirmationPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Pubcomp,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 24,
                pubcomp_reason_code: Some(PubcompPubrelCode::PacketIdentifierNotFound),
                puback_reason_code: None,
                message_id: 2,
                properties: Some(ConfirmationProperties {
                    reason_string: Some("test".to_string()),
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                }),
            },
            vec![
                112, 24, // Header
                0, 2,    // Message ID
                0x92, // reason code
                20,   // properties length
                31, 0, 4, 116, 101, 115, 116, // reasonString
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            ],
            5,
        );
    }

    #[test]
    fn test_confirmation_error_16() {
        test_decode_error(
            "Invalid pubcomp/pubrel code 16",
            vec![
                112, 4, // Header
                0, 2,  // Message ID
                16, // reason code
                0,  // properties length
            ],
        );
    }
}
