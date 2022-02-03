//! # Mqtt message decoding/encoding library
//!
//! This is a library designed to be used for creating mqtt clients or mqtt brokers.
//! As many things as were reasonable are encoded in the type system, e.g.
//! - packets have their own types, except for PUBCOMP/PUBREC/PUBREL/PUBACK, since they are essentially the same
//! - reason codes are enums and it's not possible to build a packet with an invalid reason code
//! - properties are defined for every single packet type and therefore only valid property codes can be written into the packet
//!
//! ### Supports: MQTTv3 and MQTTv5
//!
//! Messages of these versions should be decodable/encodable with this library.
//!
//! ### What works so far (Implemented and tested):
//!
//! | Encode | Decode | Packet Type |
//! |--------|--------|-------------|
//! | ✅     | ✅      | Connect     |
//! | ✅     | ✅      | Connack     |
//! | ✅     | ✅      | Publish     |
//! | ✅     | ✅      | Puback      |
//! | ✅     | ✅      | Pubrec      |
//! | ✅     | ✅      | Pubrel      |
//! | ✅     | ✅      | Pubcomp     |
//! | ✅     | ✅      | Subscribe   |
//! | ✅     | ✅      | Suback      |
//! | ✅     | ✅      | Unsubscribe |
//! | ✅     | ✅      | Unsuback    |
//! | ✅     | ✅      | Pingreq     |
//! | ✅     | ✅      | Pingresp    |
//! | ✅     | ✅      | Disconnect  |
//! | ✅     | ✅      | Auth        |
//!
//! -------------------------
//!
//! ##### However certain things still need to be added/improved:
//!
//!
//! - [ ] A better command building API?
//! - [ ] Make only necessary code public
//! - [ ] Support for Maximum Packet Size (MQTTv5). Should not send certain properties if they "bloat" the packet
//! - [ ] Ensure all properties have the correct Optionality set in their types
//! - [ ] Add some fuzzing tests to prevent unwanted panic! calls
//! - [ ] Improve documentation

pub mod auth;
pub mod byte_reader;
pub mod confirmation;
pub mod connack;
pub mod connect;
pub mod disconnect;
pub mod mqtt_writer;
pub mod packet;
pub mod publish;
pub mod structure;
pub mod suback;
pub mod subscribe;
pub mod unsubscribe;

/// Library for encoding/decoding MQTTv3 and MQTTv5 messages
///
/// # Examples
///
/// ```
/// use mqtt_packet_3_5::*;
/// let packet = MqttPacket::Pingreq(PingreqPacket {
///     fixed: FixedHeader {
///         cmd: PacketType::Pingreq,
///         qos: 0,
///         dup: false,
///         retain: false,
///     },
/// });
/// assert_eq!(Ok(vec![
///     192, 0, // Header
/// ]), mqtt_packet_3_5::PacketEncoder::encode_packet(packet, 5));
///
///
/// ```
/// # Decoder example
///
/// ```
/// use std::io;
/// let mut decoder = mqtt_packet_3_5::PacketDecoder::from_stream(io::Cursor::new(vec![192, 0])); // pingreq
/// while decoder.has_more() {
///     decoder.decode_packet(5); // will parse packets of version 5
/// }
///
///
/// ```
pub use packet::{MqttPacket, PacketDecoder, PacketEncoder};
pub use structure::*;
