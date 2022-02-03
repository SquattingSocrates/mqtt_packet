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
    }

    fn test_encode(name: &str, packet: MqttPacket, buf: Vec<u8>, protocol_version: u8) {
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
    fn test_pingreq_0() {
        let packet = MqttPacket::Pingreq;
        let buf = vec![
            192, 0, // Header
        ];
        test_decode("pingreq", packet.clone(), buf.clone(), 3);
        test_encode("pingreq", packet.clone(), buf.clone(), 3);
        test_decode("pingreq", packet.clone(), buf.clone(), 5);
        test_encode("pingreq", packet, buf, 5);
    }

    #[test]
    fn test_error_0() {
        test_decode_error(
            "Flags 1 should not be set for type Pingreq",
            vec![
                193, 0, // Header
            ],
            5,
        );
    }

    #[test]
    fn test_pingresp_0() {
        let packet = MqttPacket::Pingresp;
        let buf = vec![
            208, 0, // Header
        ];
        test_decode("pingresp", packet.clone(), buf.clone(), 3);
        test_encode("pingresp", packet, buf, 3);
    }

    #[test]
    fn test_error_1() {
        test_decode_error(
            "Flags 1 should not be set for type Pingresp",
            vec![
                209, 0, // Header
            ],
            3,
        );
    }

    #[test]
    fn test_disconnect_0() {
        let packet = MqttPacket::Disconnect(DisconnectPacket {
            reason_code: None,
            properties: None,
        });
        let buf = vec![
            224, 0, // Header
        ];
        test_decode("disconnect", packet.clone(), buf.clone(), 3);
        test_encode("disconnect", packet, buf, 3);
    }

    #[test]
    fn test_error_2() {
        test_decode_error(
            "Flags 1 should not be set for type Disconnect",
            vec![
                225, 0, // Header
            ],
            3,
        );
    }

    #[test]
    fn test_disconnect_1() {
        let packet = MqttPacket::Disconnect(DisconnectPacket {
            reason_code: Some(DisconnectCode::NormalDisconnection),
            properties: Some(DisconnectProperties {
                session_expiry_interval: Some(145),
                reason_string: Some("test".to_string()),
                user_properties: [("test".to_string(), vec!["test".to_string()])]
                    .into_iter()
                    .collect::<UserProperties>(),
                server_reference: Some("test".to_string()),
            }),
        });
        let buf = vec![
            224, 34, // Header
            0,  // reason code
            32, // properties length
            17, 0, 0, 0, 145, // sessionExpiryInterval
            31, 0, 4, 116, 101, 115, 116, // reasonString
            38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            28, 0, 4, 116, 101, 115, 116, // serverReference
        ];
        test_decode("disconnect MQTT 5", packet.clone(), buf.clone(), 5);
        test_encode("disconnect MQTT 5", packet, buf, 5);
    }

    #[test]
    fn test_disconnect_2() {
        let packet = MqttPacket::Disconnect(DisconnectPacket {
            reason_code: Some(DisconnectCode::NormalDisconnection),
            properties: None,
        });
        let buf = vec![
            224, 2, // Fixed Header (DISCONNECT, Remaining Length)
            0, // Reason Code (Normal Disconnection)
            0, // Property Length (0 => No Properties)
        ];
        test_decode(
            "disconnect MQTT 5 with no properties",
            packet.clone(),
            buf.clone(),
            5,
        );
        test_encode("disconnect MQTT 5 with no properties", packet, buf, 5);
    }

    #[test]
    fn test_error_3() {
        test_decode_error(
            "Invalid disconnect code 5",
            vec![
                224, 2,    // Fixed Header (DISCONNECT, Remaining Length)
                0x05, // Reason Code (Normal Disconnection)
                0,    // Property Length (0 => No Properties)
            ],
            5,
        );
    }
}
