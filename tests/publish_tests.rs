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

    fn test_error_encode(name: &str, packet: MqttPacket, msg: &str, protocol_version: u8) {
        println!("Failed encode error {}", name);
        assert_eq!(Err(msg.to_string()), packet.encode(protocol_version));
    }

    #[test]
    fn test_publish_0() {
        let packet = MqttPacket::Publish(PublishPacket {
            qos: 0,
            dup: false,
            retain: false,
            properties: None,
            topic: "test".to_string(),
            payload: vec![116, 101, 115, 116],
            message_id: None,
        });
        let buf = vec![
            48, 10, // Header
            0, 4, // Topic length
            116, 101, 115, 116, // Topic (test)
            116, 101, 115, 116, // Payload (test)
        ];
        test_decode("minimal publish", packet.clone(), buf.clone(), 3);
        test_encode("minimal publish", packet, buf, 3);
    }

    #[test]
    fn test_publish_1() {
        let packet = MqttPacket::Publish(PublishPacket {
            qos: 2,
            dup: true,
            retain: true,
            properties: Some(PublishProperties {
                payload_format_indicator: true,
                message_expiry_interval: Some(4321),
                topic_alias: Some(100),
                response_topic: Some("topic".to_string()),
                correlation_data: vec![1, 2, 3, 4],
                user_properties: [(
                    "test".to_string(),
                    vec!["test".to_string(), "test".to_string(), "test".to_string()],
                )]
                .into_iter()
                .collect::<UserProperties>(),
                subscription_identifiers: vec![120],
                content_type: Some("test".to_string()),
            }),
            topic: "test".to_string(),
            payload: vec![116, 101, 115, 116],
            message_id: Some(10),
        });
        let buf = vec![
            61, 86, // Header
            0, 4, // Topic length
            116, 101, 115, 116, // Topic (test)
            0, 10, // Message ID
            73, // properties length
            1, 1, // payloadFormatIndicator
            2, 0, 0, 16, 225, // message expiry interval
            35, 0, 100, // topicAlias
            8, 0, 5, 116, 111, 112, 105, 99, // response topic
            9, 0, 4, 1, 2, 3, 4, // correlationData
            38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            11, 120, // subscriptionIdentifier
            3, 0, 4, 116, 101, 115, 116, // content type
            116, 101, 115, 116, // Payload (test)
        ];
        test_decode("publish MQTT 5 properties", packet.clone(), buf.clone(), 5);
        test_encode("publish MQTT 5 properties", packet, buf, 5);
    }

    #[test]
    fn test_publish_2() {
        let packet = MqttPacket::Publish(PublishPacket {
            qos: 2,
            dup: true,
            retain: true,
            properties: Some(PublishProperties {
                payload_format_indicator: true,
                message_expiry_interval: Some(4321),
                topic_alias: Some(100),
                response_topic: Some("topic".to_string()),
                correlation_data: vec![1, 2, 3, 4],
                user_properties: [("test".to_string(), vec!["test".to_string()])]
                    .into_iter()
                    .collect::<UserProperties>(),
                subscription_identifiers: vec![120, 121, 122],
                content_type: Some("test".to_string()),
            }),
            topic: "test".to_string(),
            payload: vec![116, 101, 115, 116],
            message_id: Some(10),
        });
        let buf = vec![
            61, 64, // Header
            0, 4, // Topic length
            116, 101, 115, 116, // Topic (test)
            0, 10, // Message ID
            51, // properties length
            1, 1, // payloadFormatIndicator
            2, 0, 0, 16, 225, // message expiry interval
            35, 0, 100, // topicAlias
            8, 0, 5, 116, 111, 112, 105, 99, // response topic
            9, 0, 4, 1, 2, 3, 4, // correlationData
            38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
            11, 120, // subscriptionIdentifier
            11, 121, // subscriptionIdentifier
            11, 122, // subscriptionIdentifier
            3, 0, 4, 116, 101, 115, 116, // content type
            116, 101, 115, 116, // Payload (test)
        ];
        test_decode(
            "publish MQTT 5 with multiple same properties",
            packet.clone(),
            buf.clone(),
            5,
        );
        test_encode(
            "publish MQTT 5 with multiple same properties",
            packet,
            buf,
            5,
        );
    }

    #[test]
    fn test_publish_3() {
        let packet = MqttPacket::Publish(PublishPacket {
            qos: 2,
            dup: true,
            retain: true,
            properties: Some(PublishProperties {
                payload_format_indicator: false,
                subscription_identifiers: vec![128, 16384, 2097152],
                content_type: None,
                correlation_data: vec![],
                message_expiry_interval: None,
                response_topic: None,
                topic_alias: None,
                user_properties: UserProperties::new(),
            }),
            topic: "test".to_string(),
            payload: vec![116, 101, 115, 116],
            message_id: Some(10),
        });
        let buf = vec![
            61, 27, // Header
            0, 4, // Topic length
            116, 101, 115, 116, // Topic (test)
            0, 10, // Message ID
            14, // properties length
            1, 0, // payloadFormatIndicator
            11, 128, 1, // subscriptionIdentifier
            11, 128, 128, 1, // subscriptionIdentifier
            11, 128, 128, 128, 1, // subscriptionIdentifier
            116, 101, 115, 116, // Payload (test)
        ];
        test_decode(
            "publish MQTT 5 properties with 0-4 byte varbyte",
            packet.clone(),
            buf.clone(),
            5,
        );
        test_encode(
            "publish MQTT 5 properties with 0-4 byte varbyte",
            packet,
            buf,
            5,
        );
    }

    #[test]
    fn test_publish_4() {
        let packet = MqttPacket::Publish(PublishPacket {
            qos: 2,
            dup: true,
            retain: true,
            properties: Some(PublishProperties {
                payload_format_indicator: false,
                subscription_identifiers: vec![1, 268435455],
                content_type: None,
                correlation_data: vec![],
                message_expiry_interval: None,
                response_topic: None,
                topic_alias: None,
                user_properties: UserProperties::new(),
            }),
            topic: "test".to_string(),
            payload: vec![116, 101, 115, 116],
            message_id: Some(10),
        });
        let buf = vec![
            61, 22, // Header
            0, 4, // Topic length
            116, 101, 115, 116, // Topic (test)
            0, 10, // Message ID
            9,  // properties length
            1, 0, // payloadFormatIndicator
            11, 1, // subscriptionIdentifier
            11, 255, 255, 255, 127, // subscriptionIdentifier (max value)
            116, 101, 115, 116, // Payload (test)
        ];
        test_decode(
            "publish MQTT 5 properties with max value varbyte",
            packet.clone(),
            buf.clone(),
            5,
        );
        test_encode(
            "publish MQTT 5 properties with max value varbyte",
            packet,
            buf,
            5,
        );
    }

    #[test]
    fn test_publish_5() {
        let packet = MqttPacket::Publish(PublishPacket {
            qos: 2,
            dup: true,
            retain: true,
            properties: None,
            topic: "test".to_string(),
            payload: vec![116, 101, 115, 116],
            message_id: Some(10),
        });
        let buf = vec![
            61, 12, // Header
            0, 4, // Topic length
            116, 101, 115, 116, // Topic
            0, 10, // Message ID
            116, 101, 115, 116, // Payload
        ];
        test_decode("maximal publish", packet.clone(), buf.clone(), 3);
        test_encode("maximal publish", packet, buf, 3);
    }

    #[test]
    fn test_publish_6() {
        let packet = MqttPacket::Publish(PublishPacket {
            qos: 0,
            dup: false,
            retain: false,
            properties: None,
            topic: "test".to_string(),
            payload: vec![],
            message_id: None,
        });
        let buf = vec![
            48, 6, // Header
            0, 4, // Topic length
            116, 101, 115, 116, // Topic
                 // Empty payload
        ];
        test_decode("empty publish", packet.clone(), buf.clone(), 3);
        test_encode("empty publish", packet, buf, 3);
    }

    #[test]
    fn test_error_0() {
        test_decode_error(
            "Packet must not have both QoS bits set to 1",
            vec![
                0x36, 6, // Header
                0, 4, // Topic length
                116, 101, 115, 116, // Topic
                     // Empty payload
            ],
            3,
        );
    }
    #[test]
    fn test_publish_8() {
        test_error_encode(
            "MQTT 5.0 var byte integer >24 bits throws error",
            MqttPacket::Publish(PublishPacket {
                qos: 2,
                dup: true,
                retain: true,
                properties: Some(PublishProperties {
                    payload_format_indicator: false,
                    subscription_identifiers: vec![268435456],
                    content_type: None,
                    correlation_data: vec![],
                    message_expiry_interval: None,
                    response_topic: None,
                    topic_alias: None,
                    user_properties: UserProperties::new(),
                }),
                topic: "test".to_string(),
                payload: vec![116, 101, 115, 116],
                message_id: Some(69),
            }),
            "Invalid subscription_identifier: 268435456",
            5,
        );
    }
}
