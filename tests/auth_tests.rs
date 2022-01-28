mod tests {
    use mqtt_packet::byte_reader::*;
    use mqtt_packet::packet::*;
    use mqtt_packet::structure::*;
    use std::io::{BufReader, Cursor};

    fn dec_from_buf(v: Vec<u8>) -> PacketDecoder<Cursor<Vec<u8>>> {
        let c = Cursor::new(v);
        PacketDecoder::new(ByteReader::new(BufReader::new(c)))
    }

    fn test_encode_decode(name: &str, packet: MqttPacket, buf: Vec<u8>, protocol_version: u8) {
        let mut decoder = dec_from_buf(buf.clone());
        println!("Failed decode: {}", name);
        assert_eq!(
            packet.clone(),
            decoder.decode_packet(protocol_version).unwrap()
        );
        println!("Failed encode {}", name);
        let encoder = PacketEncoder::new();
        assert_eq!(buf, encoder.encode(packet, protocol_version).unwrap());
    }

    #[test]
    fn test_unsubscribe_0() {
        test_encode_decode(
            "auth",
            MqttPacket::Auth(AuthPacket {
                fixed: FixedHeader {
                    cmd: PacketType::Auth,
                    qos: 0,
                    dup: false,
                    retain: false,
                },
                length: 36,
                reason_code: AuthCode::Success,
                properties: Some(AuthProperties {
                    authentication_method: "test".to_string(),
                    authentication_data: Some(String::from_utf8(vec![0, 1, 2, 3]).unwrap()),
                    reason_string: Some("test".to_string()),
                    user_properties: [("test".to_string(), vec!["test".to_string()])]
                        .into_iter()
                        .collect::<UserProperties>(),
                }),
            }),
            vec![
                240, 36, // Header
                0,  // reason code
                34, // properties length
                21, 0, 4, 116, 101, 115, 116, // auth method
                22, 0, 4, 0, 1, 2, 3, // auth data
                31, 0, 4, 116, 101, 115, 116, // reasonString
                38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            ],
            5,
        );
    }
}
