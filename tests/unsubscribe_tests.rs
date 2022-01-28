mod tests {
    use mqtt_packet::byte_reader::*;
    use mqtt_packet::packet::*;
    use mqtt_packet::structure::*;
    use std::io::{BufReader, Cursor};

    fn dec_from_buf(v: Vec<u8>) -> PacketDecoder<Cursor<Vec<u8>>> {
        let c = Cursor::new(v);
        PacketDecoder::new(ByteReader::new(BufReader::new(c)))
    }

    fn test_decode(name: &str, packet: MqttPacket, buf: Vec<u8>, protocol_version: u8) {
        let mut decoder = dec_from_buf(buf.clone());
        println!("Failed: {}", name);
        assert_eq!(packet, decoder.decode_packet(protocol_version).unwrap());
    }

    fn test_encode(name: &str, packet: MqttPacket, buf: Vec<u8>, protocol_version: u8) {
        println!("Failed encode {}", name);
        let encoder = PacketEncoder::new();
        assert_eq!(buf, encoder.encode(packet, protocol_version).unwrap());
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
    fn test_unsubscribe_0() {
        let packet = MqttPacket::Unsubscribe(UnsubscribePacket {
            fixed: FixedHeader {
                cmd: PacketType::Unsubscribe,
                qos: 1,
                dup: false,
                retain: false,
            },
            length: 14,
            message_id: 7,
            properties: None,
            unsubscriptions: vec!["tfst".to_string(), "test".to_string()],
        });
        let buf = vec![
            162, 14, 0, 7, // Message ID (7)
            0, 4, // Topic length
            116, 102, 115, 116, // Topic (tfst)
            0, 4, // Topic length,
            116, 101, 115, 116, // Topic (test)
        ];
        test_decode("unsubscribe", packet.clone(), buf.clone(), 3);
        test_encode("unsubscribe", packet, buf, 3);
    }

    #[test]
    fn test_error_0() {
        test_decode_error(
            "Invalid header flag bits, must be 0x2 for Unsubscribe packet",
            vec![
                160, 14, 0, 7, // Message ID (7)
                0, 4, // Topic length
                116, 102, 115, 116, // Topic (tfst)
                0, 4, // Topic length,
                116, 101, 115, 116, // Topic (test)
            ],
            3,
        );
    }

    #[test]
    fn test_unsubscribe_1() {
        let packet = MqttPacket::Unsubscribe(UnsubscribePacket {
            fixed: FixedHeader {
                cmd: PacketType::Unsubscribe,
                qos: 1,
                dup: false,
                retain: false,
            },
            length: 28,
            message_id: 7,
            properties: Some(UnsubscribeProperties {
                user_properties: vec![("test".to_string(), vec!["test".to_string()])]
                    .into_iter()
                    .collect::<UserProperties>(),
            }),
            unsubscriptions: vec!["tfst".to_string(), "test".to_string()],
        });
        let buf = vec![
            162, 28, 0, 7,  // Message ID (7)
            13, // properties length
            38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            0, 4, // Topic length
            116, 102, 115, 116, // Topic (tfst)
            0, 4, // Topic length,
            116, 101, 115, 116, // Topic (test)
        ];
        test_decode("unsubscribe MQTT 5", packet.clone(), buf.clone(), 5);
        test_encode("unsubscribe MQTT 5", packet, buf, 5);
    }

    #[test]
    fn test_unsuback_0() {
        let packet = MqttPacket::Unsuback(UnsubackPacket {
            fixed: FixedHeader {
                cmd: PacketType::Unsuback,
                qos: 0,
                dup: false,
                retain: false,
            },
            length: 2,
            message_id: 8,
            properties: None,
            granted: vec![],
        });
        let buf = vec![
            176, 2, // Header
            0, 8, // Message ID
        ];
        test_decode("unsuback", packet.clone(), buf.clone(), 3);
        test_encode("unsuback", packet, buf, 3);
    }

    #[test]
    fn test_error_1() {
        test_decode_error(
            "Flags 1 should not be set for type Unsuback",
            vec![
                177, 2, // Header
                0, 8, // Message ID
            ],
            3,
        );
    }

    #[test]
    fn test_unsuback_1() {
        let packet = MqttPacket::Unsuback(UnsubackPacket {
            fixed: FixedHeader {
                cmd: PacketType::Unsuback,
                qos: 0,
                dup: false,
                retain: false,
            },
            length: 25,
            message_id: 8,
            properties: Some(ConfirmationProperties {
                reason_string: Some("test".to_string()),
                user_properties: vec![("test".to_string(), vec!["test".to_string()])]
                    .into_iter()
                    .collect::<UserProperties>(),
            }),
            granted: vec![UnsubackCode::Success, UnsubackCode::UnspecifiedError],
        });
        let buf = vec![
            176, 25, // Header
            0, 8,  // Message ID
            20, // properties length
            31, 0, 4, 116, 101, 115, 116, // reasonString
            38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            0, 128, // success and error
        ];
        test_decode("unsuback MQTT 5", packet.clone(), buf.clone(), 5);
        test_encode("unsuback MQTT 5", packet, buf, 5);
    }

    #[test]
    fn test_error_2() {
        test_decode_error(
            "Invalid unsuback code 132",
            vec![
                176, 4, // Header
                0, 8,    // Message ID
                0,    // properties length
                0x84, // reason codes
            ],
            5,
        );
    }
}
