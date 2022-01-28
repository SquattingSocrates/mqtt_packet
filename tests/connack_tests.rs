use mqtt_packet_3_5::byte_reader::*;
use mqtt_packet_3_5::packet::*;
use mqtt_packet_3_5::structure::*;
use std::io::{BufReader, Cursor};

fn dec_from_buf(v: Vec<u8>) -> PacketDecoder<Cursor<Vec<u8>>> {
  let c = Cursor::new(v);
  PacketDecoder::new(ByteReader::new(BufReader::new(c)))
}

fn test_decode(name: &str, packet: ConnackPacket, buf: Vec<u8>, protocol_version: u8) {
  let mut decoder = dec_from_buf(buf);
  println!("Failed: {}", name);
  assert_eq!(
    MqttPacket::Connack(packet),
    decoder.decode_packet(protocol_version).unwrap()
  );
}

fn test_encode(name: &str, packet: ConnackPacket, buf: Vec<u8>, protocol_version: u8) {
  let encoder = PacketEncoder::new();
  println!("Failed: {}", name);
  assert_eq!(
    buf,
    encoder.encode_connack(packet, protocol_version).unwrap()
  );
}

fn test_parse_error(name: &str, msg: String, buf: Vec<u8>) {
  println!("Failed: {}", name);
  let mut decoder = dec_from_buf(buf);
  assert_eq!(Err(msg), decoder.decode_packet(3));
}

#[test]
fn test_connack_v4() {
  test_decode(
    "Version 4 CONNACK",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 2,
      properties: None,
      reason_code: None,
      return_code: Some(1),
      session_present: false,
    },
    vec![
      32, 2, // Fixed Header (CONNACK, Remaining Length)
      0,
      1, // Variable Header (Session not present, Connection Refused - unacceptable protocol version)
    ],
    4,
  );
  test_encode(
    "Version 4 CONNACK",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 2,
      properties: None,
      reason_code: None,
      return_code: Some(1),
      session_present: false,
    },
    vec![
      32, 2, // Fixed Header (CONNACK, Remaining Length)
      0,
      1, // Variable Header (Session not present, Connection Refused - unacceptable protocol version)
    ],
    4,
  );
}

#[test]
fn test_connack_v5() {
  test_decode(
    "Version 5 CONNACK",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 3,
      properties: None,
      reason_code: Some(140),
      return_code: None,
      session_present: false,
    },
    vec![
      32, 3, // Fixed Header (CONNACK, Remaining Length)
      0, 140, // Variable Header (Session not present, Bad authentication method)
      0,   // Property Length Zero
    ],
    5,
  );
  test_encode(
    "Version 5 CONNACK",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 3,
      properties: None,
      reason_code: Some(140),
      return_code: None,
      session_present: false,
    },
    vec![
      32, 3, // Fixed Header (CONNACK, Remaining Length)
      0, 140, // Variable Header (Session not present, Bad authentication method)
      0,   // Property Length Zero
    ],
    5,
  );
}
#[test]
fn test_connack_v4_v5_mode() {
  test_decode(
    "Version 4 CONNACK in Version 5 mode",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 2,
      properties: None,
      reason_code: Some(1),
      return_code: None,
      session_present: false,
    },
    vec![
      32, 2, // Fixed Header (CONNACK, Remaining Length)
      0,
      1, // Variable Header (Session not present, Connection Refused - unacceptable protocol version)
    ],
    5,
  );
}

#[test]
fn test_connack_7() {
  test_decode(
    "connack with return code 0",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 2,
      session_present: false,
      reason_code: None,
      return_code: Some(0),
      properties: None,
    },
    vec![32, 2, 0, 0],
    3,
  );
}

#[test]
fn test_connack_8() {
  test_decode(
    "connack MQTT 5 with properties",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 87,
      session_present: false,
      reason_code: Some(0),
      return_code: None,
      properties: Some(ConnackProperties {
        is_default: false,
        session_expiry_interval: 1234,
        receive_maximum: 432,
        maximum_qos: 2,
        retain_available: true,
        maximum_packet_size: Some(100),
        assigned_client_identifier: Some("test".to_string()),
        topic_alias_maximum: 456,
        reason_string: Some("test".to_string()),
        user_properties: [("test".to_string(), vec!["test".to_string()])]
          .into_iter()
          .collect::<UserProperties>(),
        wildcard_subscription_available: true,
        subscription_identifiers_available: true,
        shared_subscription_available: false,
        server_keep_alive: Some(1234),
        response_information: Some("test".to_string()),
        server_reference: Some("test".to_string()),
        authentication_method: Some("test".to_string()),
        authentication_data: Some(String::from_utf8(vec![1, 2, 3, 4]).unwrap()),
      }),
    },
    vec![
      32, 87, 0, 0, 84, // properties length
      17, 0, 0, 4, 210, // sessionExpiryInterval
      33, 1, 176, // receiveMaximum
      36, 2, // Maximum qos
      37, 1, // retainAvailable
      39, 0, 0, 0, 100, // maximumPacketSize
      18, 0, 4, 116, 101, 115, 116, // assignedClientIdentifier
      34, 1, 200, // topicAliasMaximum
      31, 0, 4, 116, 101, 115, 116, // reasonString
      38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, // userProperties
      40, 1, // wildcardSubscriptionAvailable
      41, 1, // subscriptionIdentifiersAvailable
      42, 0, // sharedSubscriptionAvailable
      19, 4, 210, // serverKeepAlive
      26, 0, 4, 116, 101, 115, 116, // responseInformation
      28, 0, 4, 116, 101, 115, 116, // serverReference
      21, 0, 4, 116, 101, 115, 116, // authenticationMethod
      22, 0, 4, 1, 2, 3, 4, // authenticationData
    ],
    5,
  )
}

#[test]
fn test_connack_9() {
  test_decode(
    "connack MQTT 5 with properties and doubled user properties",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 100,
      session_present: false,
      reason_code: Some(0),
      return_code: None,
      properties: Some(ConnackProperties {
        is_default: false,
        session_expiry_interval: 1234,
        receive_maximum: 432,
        maximum_qos: 2,
        retain_available: true,
        maximum_packet_size: Some(100),
        assigned_client_identifier: Some("test".to_string()),
        topic_alias_maximum: 456,
        reason_string: Some("test".to_string()),
        user_properties: [(
          "test".to_string(),
          vec!["test".to_string(), "test".to_string()],
        )]
        .into_iter()
        .collect::<UserProperties>(),
        wildcard_subscription_available: true,
        subscription_identifiers_available: true,
        shared_subscription_available: false,
        server_keep_alive: Some(1234),
        response_information: Some("test".to_string()),
        server_reference: Some("test".to_string()),
        authentication_method: Some("test".to_string()),
        authentication_data: Some(String::from_utf8(vec![1, 2, 3, 4]).unwrap()),
      }),
    },
    vec![
      32, 100, 0, 0, 97, // properties length
      17, 0, 0, 4, 210, // sessionExpiryInterval
      33, 1, 176, // receiveMaximum
      36, 2, // Maximum qos
      37, 1, // retainAvailable
      39, 0, 0, 0, 100, // maximumPacketSize
      18, 0, 4, 116, 101, 115, 116, // assignedClientIdentifier
      34, 1, 200, // topicAliasMaximum
      31, 0, 4, 116, 101, 115, 116, // reasonString
      38, 0, 4, 116, 101, 115, 116, 0, 4, 116, 101, 115, 116, 38, 0, 4, 116, 101, 115, 116, 0, 4,
      116, 101, 115, 116, // userProperties
      40, 1, // wildcardSubscriptionAvailable
      41, 1, // subscriptionIdentifiersAvailable
      42, 0, // sharedSubscriptionAvailable
      19, 4, 210, // serverKeepAlive
      26, 0, 4, 116, 101, 115, 116, // responseInformation
      28, 0, 4, 116, 101, 115, 116, // serverReference
      21, 0, 4, 116, 101, 115, 116, // authenticationMethod
      22, 0, 4, 1, 2, 3, 4, // authenticationData
    ],
    5,
  );
}

#[test]
fn test_connack_10() {
  test_decode(
    "connack with return code 0 session present bit set",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 2,
      session_present: true,
      reason_code: None,
      return_code: Some(0),
      properties: None,
    },
    vec![32, 2, 1, 0],
    3,
  );
}

#[test]
fn test_connack_11() {
  test_decode(
    "connack with return code 5",
    ConnackPacket {
      fixed: FixedHeader {
        cmd: PacketType::Connack,
        qos: 0,
        dup: false,
        retain: false,
      },
      length: 2,
      session_present: false,
      reason_code: None,
      return_code: Some(5),
      properties: None,
    },
    vec![32, 2, 0, 5],
    3,
  );
}

// ==========================
// Test error cases
// ==========================

#[test]
fn test_connack_err() {
  //   Where a flag bit is marked as “Reserved” in Table 2.2 - Flag Bits, it is reserved for future use and MUST be set to the value listed in that table [MQTT-2.2.2-1]. If invalid flags are received, the receiver MUST close the Network Connection [MQTT-2.2.2-2]
  test_parse_error(
    "Invalid header flag bits, must be 0x0 for connack packet",
    "Flags 1 should not be set for type Connack".to_string(),
    vec![
      33, 2, // header
      0, // flags
      5, // return code
    ],
  )
}

#[test]
fn test_connack_err_1() {
  // Byte 1 is the "Connect Acknowledge Flags". Bits 7-1 are reserved and MUST be set to 0 [MQTT-3.2.2-1].
  test_parse_error(
    "Invalid connack flags, bits 7-1 must be set to 0",
    "Invalid connack flags, bits 7-1 must be set to 0".to_string(),
    vec![
      32, 2, // header
      2, // flags
      5, // return code
    ],
  )
}
