use crate::byte_reader::ByteReader;
use crate::structure::*;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MqttPacket {
    Connect(ConnectPacket),
    Connack(ConnackPacket),
    Subscribe(SubscribePacket),
    Suback(SubackPacket),
    Publish(PublishPacket),
    Puback(ConfirmationPacket),
    Pubrec(ConfirmationPacket),
    Pubrel(ConfirmationPacket),
    Pubcomp(ConfirmationPacket),
    Unsubscribe(UnsubscribePacket),
    Unsuback(UnsubackPacket),
    Pingreq(PingreqPacket),
    Pingresp(PingrespPacket),
    Disconnect(DisconnectPacket),
    Auth(AuthPacket),
}

pub struct PacketDecoder<R: io::Read> {
    pub reader: ByteReader<R>,
}

impl<R: io::Read> PacketDecoder<R> {
    pub fn new(reader: ByteReader<R>) -> PacketDecoder<R> {
        PacketDecoder { reader }
    }

    /// Creates a new decoder and binds it to a stream
    ///
    /// # Examples
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
    pub fn from_stream(src: R) -> PacketDecoder<R> {
        PacketDecoder::new(ByteReader::new(io::BufReader::new(src)))
    }

    /// Creates a new decoder and binds it to a BufReader
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io;
    /// let buf = io::BufReader::new(io::Cursor::new(vec![192, 0])); // pingreq
    /// let mut decoder = mqtt_packet_3_5::PacketDecoder::from_bufreader(buf);
    /// while decoder.has_more() {
    ///     decoder.decode_packet(5); // will parse packets of version 5
    /// }
    ///
    ///
    /// ```
    pub fn from_bufreader(buf: io::BufReader<R>) -> PacketDecoder<R> {
        PacketDecoder::new(ByteReader::new(buf))
    }

    /// Decodes MQTT messages from an underlying readable
    ///
    /// If an error happens the decoder tries to get the packet length (variable length in in position 1-4)
    /// and discard `length` bytes. It's up to the user of this crate to close connections/streams
    /// that deliver invalid data if necessary
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io;
    /// let buf = io::BufReader::new(io::Cursor::new(vec![
    ///        16, 18, // Header
    ///        0, 6, // Protocol ID length
    ///        77, 81, 73, 115, 100, 112, // Protocol ID
    ///        3,   // Protocol version
    ///        0,   // Connect flags
    ///        0, 30, // Keepalive
    ///        0, 4, // Client ID length
    ///        116, 101, 115, 116, // Client ID
    ///        // new packet here
    ///        192, 0
    /// ])); // pingreq
    /// let mut decoder = mqtt_packet_3_5::PacketDecoder::from_bufreader(buf);
    /// let mut protocol_version = 5;
    /// while decoder.has_more() {
    ///     let msg = decoder.decode_packet(protocol_version); // will parse packets of version 5
    ///     // set the protocol version for a client on a stream
    ///     if let Ok(mqtt_packet_3_5::MqttPacket::Connect(packet)) = msg {
    ///         protocol_version = packet.protocol_version;
    ///     }
    /// }
    ///
    /// ```
    pub fn decode_packet(&mut self, protocol_version: u8) -> Res<MqttPacket> {
        let (length, fixed) = self.reader.read_header()?;
        let dec = self.decode_by_type(fixed, length, protocol_version);
        if dec.is_err() {
            // TODO: this should probably return an Error that indicates some
            // critical failure
            self.reader.consume()?;
        }
        self.reader.reset_limit();
        dec
    }

    pub fn has_more(&mut self) -> bool {
        self.reader.has_more()
    }

    fn decode_by_type(
        &mut self,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<MqttPacket> {
        // let reader = self.reader.take(length);
        Ok(match fixed.cmd {
            PacketType::Connect => {
                // passing protocol_version is unnecessary here
                MqttPacket::Connect(ConnectPacket::decode(&mut self.reader, fixed, length, 5)?)
            }
            PacketType::Connack => MqttPacket::Connack(self.decode_connack_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Subscribe => MqttPacket::Subscribe(SubscribePacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Suback => MqttPacket::Suback(SubackPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Publish => MqttPacket::Publish(PublishPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Puback => MqttPacket::Puback(ConfirmationPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubrec => MqttPacket::Pubrec(ConfirmationPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubrel => MqttPacket::Pubrel(ConfirmationPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubcomp => MqttPacket::Pubcomp(ConfirmationPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Unsubscribe => MqttPacket::Unsubscribe(UnsubscribePacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Unsuback => MqttPacket::Unsuback(UnsubackPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pingreq => MqttPacket::Pingreq(PingreqPacket),
            PacketType::Pingresp => MqttPacket::Pingresp(PingrespPacket),
            PacketType::Disconnect => MqttPacket::Disconnect(DisconnectPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Auth => MqttPacket::Auth(AuthPacket::decode(
                &mut self.reader,
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Reserved => return Err("Cannot use RESERVED message type".to_string()),
        })
    }
}

#[derive(Default)]
pub struct PacketEncoder {
    pub(crate) buf: Vec<u8>,
}

impl PacketEncoder {
    pub fn new() -> PacketEncoder {
        PacketEncoder { buf: vec![] }
    }

    /// Encodes any MqttPacket
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
    pub fn encode_packet(packet: MqttPacket, protocol_version: u8) -> Res<Vec<u8>> {
        PacketEncoder::new().encode(packet, protocol_version)
    }

    pub fn encode(self, packet: MqttPacket, protocol_version: u8) -> Res<Vec<u8>> {
        match packet {
            MqttPacket::Puback(packet)
            | MqttPacket::Pubrec(packet)
            | MqttPacket::Pubrel(packet)
            | MqttPacket::Pubcomp(packet) => packet.encode(protocol_version),
            MqttPacket::Suback(packet) => packet.encode(protocol_version),
            MqttPacket::Subscribe(packet) => packet.encode(protocol_version),
            MqttPacket::Publish(packet) => packet.encode(protocol_version),
            MqttPacket::Connect(packet) => packet.encode(protocol_version),
            MqttPacket::Connack(packet) => packet.encode(protocol_version),
            MqttPacket::Unsubscribe(packet) => packet.encode(protocol_version),
            MqttPacket::Unsuback(packet) => packet.encode(protocol_version),
            MqttPacket::Disconnect(packet) => packet.encode(protocol_version),
            MqttPacket::Pingreq(packet) => packet.encode(protocol_version),
            MqttPacket::Pingresp(packet) => packet.encode(protocol_version),
            MqttPacket::Auth(packet) => packet.encode(protocol_version),
        }
    }

    fn write_empty(mut self, fixed: FixedHeader) -> Res<Vec<u8>> {
        self.buf.push(FixedHeader::encode(&fixed));
        self.buf.push(0);
        Ok(self.buf)
    }

    pub fn encode_multibyte_num(message_id: u32) -> Vec<u8> {
        vec![(message_id >> 8) as u8, message_id as u8]
    }

    pub fn encode_variable_num(mut length: u32) -> Vec<u8> {
        let mut v = Vec::<u8>::with_capacity(4);
        while length > 0 {
            let mut next = length % 128;
            length /= 128;
            if length > 0 {
                next |= 0x80;
            }
            v.push(next as u8);
        }
        v
    }

    pub fn write_variable_num(&mut self, num: u32) -> Res<()> {
        if num > VARBYTEINT_MAX {
            return Err(format!("Invalid variable int {}", num));
        }
        let mut encoded = if num == 0 {
            vec![0]
        } else {
            Self::encode_variable_num(num)
        };
        self.buf.append(&mut encoded);
        Ok(())
    }

    pub fn write_utf8_string(&mut self, s: String) {
        self.write_u16(s.len() as u16);
        for b in s.bytes() {
            self.buf.push(b);
        }
    }

    /// a Binary vector should never be empty
    pub fn write_binary(&mut self, s: Vec<u8>) {
        self.write_u16(s.len() as u16);
        self.write_vec(s);
    }

    pub fn write_u16(&mut self, length: u16) {
        self.buf.push((length >> 8) as u8);
        self.buf.push(length as u8);
    }

    pub fn write_u32(&mut self, num: u32) {
        let mut encoded = vec![
            (num >> 24) as u8,
            (num >> 16) as u8,
            (num >> 8) as u8,
            num as u8,
        ];
        self.buf.append(&mut encoded);
    }

    pub fn write_header(&mut self, fixed: FixedHeader) {
        self.buf.push(fixed.encode());
    }

    pub fn write_u8(&mut self, byte: u8) {
        self.buf.push(byte);
    }

    pub fn write_vec(&mut self, mut v: Vec<u8>) {
        self.buf.append(&mut v);
    }
}

// pub struct PropertyEncoder {
//     writer: PacketEncoder,
// }
// impl PropertyEncoder {
//     fn new() -> PropertyEncoder {
//         PropertyEncoder {
//             writer: PacketEncoder::new(),
//         }
//     }

//     pub(crate) fn encode<T: Properties<T>>(props: Option<T>, protocol_version: u8) -> Res<Vec<u8>> {
//         // Confirm should not add empty property length with no properties (rfc 3.4.2.2.1)
//         if protocol_version == 5 {
//             if props.is_some() {
//                 let pairs = props.unwrap().to_pairs()?;
//                 let mut v = PropertyEncoder::new().write_properties(pairs)?;
//                 // dirty hack
//                 for b in PacketEncoder::encode_variable_num(v.len() as u32) {
//                     v.insert(0, b);
//                 }
//                 Ok(v)
//             } else {
//                 Ok(vec![0]) // empty properties
//             }
//         } else {
//             Ok(vec![]) // no properties exist in MQTT < 5
//         }
//     }

//     pub fn write_properties(mut self, props: Vec<(u8, PropType)>) -> Res<Vec<u8>> {
//         for prop in props {
//             match prop {
//                 (code, PropType::U32(v)) => {
//                     self.writer.write_u8(code);
//                     self.writer.write_u32(v);
//                 }
//                 (code, PropType::U16(v)) => {
//                     self.writer.write_u8(code);
//                     self.writer.write_u16(v)
//                 }
//                 (code, PropType::U8(v)) => {
//                     self.writer.write_u8(code);
//                     self.writer.write_u8(v)
//                 }
//                 (code, PropType::String(v)) => {
//                     self.writer.write_u8(code);
//                     self.writer.write_utf8_string(v)
//                 }
//                 (code, PropType::Binary(v)) => {
//                     self.writer.write_u8(code);
//                     self.writer.write_binary(v)
//                 }
//                 // should never happen actually
//                 (_, PropType::Pair(_, _)) => {}
//                 // write code code and two strings for each key-value
//                 // pair
//                 (code, PropType::Map(map)) => {
//                     for (k, v) in map.into_iter() {
//                         // split into pairs
//                         for val in v {
//                             self.writer.write_u8(code);
//                             self.writer.write_utf8_string(k.to_string());
//                             self.writer.write_utf8_string(val);
//                         }
//                     }
//                 }
//                 (code, PropType::VarInt(num)) => {
//                     self.writer.write_u8(code);
//                     self.writer.write_variable_num(num)?;
//                 }
//                 (code, PropType::Bool(v)) => {
//                     self.writer.write_u8(code);
//                     self.writer.write_u8(v as u8)
//                 }
//                 (code, PropType::U32Vec(v)) => {
//                     self.writer.write_u8(code);
//                     for num in v {
//                         self.writer.write_u32(num);
//                     }
//                 }
//             }
//         }
//         Ok(self.writer.buf)
//     }
// }
