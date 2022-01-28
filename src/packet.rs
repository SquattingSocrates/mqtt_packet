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
    pub(crate) reader: ByteReader<R>,
}

impl<R: io::Read> PacketDecoder<R> {
    pub fn new(reader: ByteReader<R>) -> PacketDecoder<R> {
        PacketDecoder { reader }
    }

    pub fn decode_packet(&mut self, protocol_version: u8) -> Res<MqttPacket> {
        let (length, fixed) = self.reader.read_header()?;
        println!(
            "GOT LENGTH AND HEADER {:?} {:?}. Version: {}",
            length, fixed, protocol_version
        );
        let dec = self.decode_by_type(fixed, length, protocol_version);
        if let Err(_) = &dec {
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
        Ok(match fixed.cmd {
            PacketType::Connect => {
                MqttPacket::Connect(self.decode_connect_with_length(fixed, length)?)
            }
            PacketType::Connack => MqttPacket::Connack(self.decode_connack_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Subscribe => MqttPacket::Subscribe(self.decode_subscribe_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Suback => MqttPacket::Suback(self.decode_suback_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Publish => MqttPacket::Publish(self.decode_publish_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Puback => MqttPacket::Puback(self.decode_confirmation_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubrec => MqttPacket::Pubrec(self.decode_confirmation_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubrel => MqttPacket::Pubrel(self.decode_confirmation_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pubcomp => MqttPacket::Pubcomp(self.decode_confirmation_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Unsubscribe => MqttPacket::Unsubscribe(
                self.decode_unsubscribe_with_length(fixed, length, protocol_version)?,
            ),
            PacketType::Unsuback => MqttPacket::Unsuback(self.decode_unsuback_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Pingreq => MqttPacket::Pingreq(PingreqPacket { fixed }),
            PacketType::Pingresp => MqttPacket::Pingresp(PingrespPacket { fixed }),
            PacketType::Disconnect => MqttPacket::Disconnect(self.decode_disconnect_with_length(
                fixed,
                length,
                protocol_version,
            )?),
            PacketType::Auth => {
                MqttPacket::Auth(self.decode_auth_with_length(fixed, length, protocol_version)?)
            }
            PacketType::Reserved => return Err("Cannot use RESERVED message type".to_string()),
        })
    }
}

pub struct PacketEncoder {
    //<W: io::Write> {
    // pub(crate) writer: W,
    pub(crate) buf: Vec<u8>,
}

impl PacketEncoder {
    pub fn new() -> PacketEncoder {
        PacketEncoder { buf: vec![] }
    }

    pub fn encode(mut self, packet: MqttPacket, protocol_version: u8) -> Res<Vec<u8>> {
        match packet {
            MqttPacket::Puback(packet)
            | MqttPacket::Pubrec(packet)
            | MqttPacket::Pubrel(packet)
            | MqttPacket::Pubcomp(packet) => self.encode_confirmation(packet, protocol_version),
            MqttPacket::Suback(packet) => self.encode_suback(packet, protocol_version),
            MqttPacket::Subscribe(packet) => self.encode_subscribe(packet, protocol_version),
            MqttPacket::Publish(packet) => self.encode_publish(packet, protocol_version),
            MqttPacket::Connect(packet) => self.encode_connect(packet),
            MqttPacket::Connack(packet) => self.encode_connack(packet, protocol_version),
            MqttPacket::Unsubscribe(packet) => self.encode_unsubscribe(packet, protocol_version),
            MqttPacket::Unsuback(packet) => self.encode_unsuback(packet, protocol_version),
            MqttPacket::Disconnect(packet) => self.encode_disconnect(packet, protocol_version),
            MqttPacket::Pingreq(packet) => self.write_empty(packet.fixed),
            MqttPacket::Pingresp(packet) => self.write_empty(packet.fixed),
            MqttPacket::Auth(packet) => self.encode_auth(packet, protocol_version),
        }
    }

    fn write_empty(mut self, fixed: FixedHeader) -> Res<Vec<u8>> {
        self.buf.push(FixedHeader::encode(&fixed));
        self.buf.push(0);
        Ok(self.buf)
    }

    pub fn encode_multibyte_num(message_id: u32) -> Vec<u8> {
        // println!("SPLITTING MESSAGE_ID {}", message_id, message_id >> 8, message_id as u8);
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

pub struct PropertyEncoder {
    writer: PacketEncoder,
}
impl PropertyEncoder {
    fn new() -> PropertyEncoder {
        PropertyEncoder {
            writer: PacketEncoder::new(),
        }
    }

    pub(crate) fn encode<T: Properties<T>>(props: Option<T>, protocol_version: u8) -> Res<Vec<u8>> {
        // Confirm should not add empty property length with no properties (rfc 3.4.2.2.1)
        if protocol_version == 5 {
            if props.is_some() {
                let pairs = props.unwrap().to_pairs()?;
                let mut v = PropertyEncoder::new().write_properties(pairs)?;
                // dirty hack
                for b in PacketEncoder::encode_variable_num(v.len() as u32) {
                    v.insert(0, b);
                }
                Ok(v)
            } else {
                Ok(vec![0]) // empty properties
            }
        } else {
            Ok(vec![]) // no properties exist in MQTT < 5
        }
    }

    pub fn write_properties(mut self, props: Vec<(u8, PropType)>) -> Res<Vec<u8>> {
        for prop in props {
            match prop {
                (code, PropType::U32(v)) => {
                    self.writer.write_u8(code);
                    self.writer.write_u32(v);
                }
                (code, PropType::U16(v)) => {
                    self.writer.write_u8(code);
                    self.writer.write_u16(v)
                }
                (code, PropType::U8(v)) => {
                    self.writer.write_u8(code);
                    self.writer.write_u8(v)
                }
                (code, PropType::String(v)) => {
                    self.writer.write_u8(code);
                    self.writer.write_utf8_string(v)
                }
                (code, PropType::Binary(v)) => {
                    self.writer.write_u8(code);
                    self.writer.write_binary(v)
                }
                // should never happen actually
                (_, PropType::Pair(_, _)) => {}
                // write code code and two strings for each key-value
                // pair
                (code, PropType::Map(map)) => {
                    for (k, v) in map.into_iter() {
                        // split into pairs
                        for val in v {
                            self.writer.write_u8(code);
                            self.writer.write_utf8_string(k.to_string());
                            self.writer.write_utf8_string(val);
                        }
                    }
                }
                (code, PropType::VarInt(num)) => {
                    self.writer.write_u8(code);
                    self.writer.write_variable_num(num)?;
                }
                (code, PropType::Bool(v)) => {
                    self.writer.write_u8(code);
                    self.writer.write_u8(v as u8)
                }
                (code, PropType::U32Vec(v)) => {
                    self.writer.write_u8(code);
                    for num in v {
                        self.writer.write_u32(num);
                    }
                }
            }
        }
        Ok(self.writer.buf)
    }
}
