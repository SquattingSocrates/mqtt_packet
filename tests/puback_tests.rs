mod tests {
    use mqtt_packet::byte_reader::*;
    use mqtt_packet::connect::*;
    use mqtt_packet::packet::*;
    use mqtt_packet::structure::*;
    use std::io::{BufReader, Cursor};

    fn dec_from_buf(v: Vec<u8>) -> PacketDecoder<Cursor<Vec<u8>>> {
        let c = Cursor::new(v);
        PacketDecoder::new(ByteReader::new(BufReader::new(c)))
    }

    fn test_decode(name: &str, packet: ConfirmationPacket, buf: Vec<u8>) {
        let mut decoder = dec_from_buf(buf.clone());
        println!("Failed: {}", name);
        assert_eq!(
            MqttPacket::Puback(packet.clone()),
            decoder.decode_packet(5).unwrap()
        );
    }

    fn test_encode(name: &str, packet: ConfirmationPacket, buf: Vec<u8>) {
        let encoder = PacketEncoder::new();
        assert_eq!(buf, encoder.encode_confirmation(packet, 5).unwrap());
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
                reason_code: Some(0),
                properties: None,
                message_id: 42,
            },
            vec![
                64, 2, // Fixed Header (PUBACK, Remaining Length)
                0,
                42, // Variable Header (2 Bytes: Packet Identifier 42, Implied Reason code: Success, Implied no properties)
            ],
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
            reason_code: Some(0),
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
                reason_code: Some(0),
                properties: None,
                message_id: 42,
            },
            vec![
                64, 4, // Fixed Header (PUBACK, Remaining Length)
                0, 42,
                0, // Variable Header (2 Bytes: Packet Identifier 42, Reason code: 0 Success)
                0, // no properties
            ],
        );
    }
}
