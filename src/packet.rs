use crate::byte_reader::ByteReader;
use crate::connect::ConnectFlags;
use crate::structure::*;
use std::io;

#[derive(Debug, PartialEq)]
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
        println!("GOT LENGTH AND HEADER {:?} {:?}", length, fixed);
        self.reader.take(length);
        match self.decode_by_type(fixed, length, protocol_version) {
            Ok(msg) => {
                self.reader.reset_limit();
                Ok(msg)
            }
            Err(e) => {
                self.reader.reset_limit();
                Err(e)
            }
        }
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

    pub fn write_variable_num(&mut self, length: u32) {
        let mut encoded = Self::encode_variable_num(length);
        self.buf.append(&mut encoded);
    }

    pub fn write_utf8_string(&mut self, s: String) {
        self.write_u16(s.len() as u16);
        for b in s.bytes() {
            self.buf.push(b);
        }
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

    pub(crate) fn encode<T: Properties>(props: Option<T>, protocol_version: u8) -> Res<Vec<u8>> {
        // Confirm should not add empty property length with no properties (rfc 3.4.2.2.1)
        if protocol_version == 5 {
            if props.is_some() {
                let pairs = props.unwrap().to_pairs()?;
                PropertyEncoder::new().write_properties(pairs)
            } else {
                Ok(vec![0]) // empty properties
            }
        } else {
            Ok(vec![]) // no properties exist in MQTT < 5
        }
    }

    pub fn write_properties(mut self, props: Vec<(u8, PropType)>) -> Res<Vec<u8>> {
        self.writer.write_variable_num(props.len() as u32);
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
                // should never happen actually
                (_, PropType::Pair(_, _)) => {}
                (code, PropType::Map(map)) => {
                    self.writer.write_u8(code);
                    for (k, v) in map.into_iter() {
                        for val in v {
                            self.writer.write_u8(code);
                            self.writer.write_utf8_string(val);
                        }
                    }
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
