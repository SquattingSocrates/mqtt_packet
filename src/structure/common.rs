use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type UserProperties = HashMap<String, Vec<String>>;
pub type Res<T> = Result<T, String>;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Mqtt,
    MQIsdp,
}

impl Protocol {
    pub fn from_source(s: &str) -> Res<Protocol> {
        Ok(match s {
            "MQIsdp" => Protocol::MQIsdp,
            "MQTT" => Protocol::Mqtt,
            s => return Err(format!("Invalid protocolId {}", s)),
        })
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum QoS {
    QoS0,
    QoS1,
    QoS2,
}

impl QoS {
    pub fn to_byte(&self) -> u8 {
        match self {
            QoS::QoS0 => 0,
            QoS::QoS1 => 1,
            QoS::QoS2 => 2,
        }
    }
}

static DUP_MASK: u8 = 0x8;
static QOS_MASK: u8 = 0x6;
static QOS_SHIFT: u8 = 1;
static RETAIN_MASK: u8 = 0x1;
pub static VARBYTEINT_MAX: u32 = 268435455;

impl PacketType {
    pub fn from_bits(bits: u8) -> PacketType {
        match bits {
            0 => PacketType::Reserved,
            1 => PacketType::Connect,
            2 => PacketType::Connack,
            3 => PacketType::Publish,
            4 => PacketType::Puback,
            5 => PacketType::Pubrec,
            6 => PacketType::Pubrel,
            7 => PacketType::Pubcomp,
            8 => PacketType::Subscribe,
            9 => PacketType::Suback,
            10 => PacketType::Unsubscribe,
            11 => PacketType::Unsuback,
            12 => PacketType::Pingreq,
            13 => PacketType::Pingresp,
            14 => PacketType::Disconnect,
            15 => PacketType::Auth,
            _ => PacketType::Reserved,
        }
    }

    fn to_bits(t: PacketType) -> u8 {
        match t {
            PacketType::Reserved => 0x0,
            PacketType::Connect => 0x10,
            PacketType::Connack => 0x20,
            PacketType::Publish => 0x30,
            PacketType::Puback => 0x40,
            PacketType::Pubrec => 0x50,
            PacketType::Pubrel => 0x60,
            PacketType::Pubcomp => 0x70,
            PacketType::Subscribe => 0x80,
            PacketType::Suback => 0x90,
            PacketType::Unsubscribe => 0xA0,
            PacketType::Unsuback => 0xB0,
            PacketType::Pingreq => 0xC0,
            PacketType::Pingresp => 0xD0,
            PacketType::Disconnect => 0xE0,
            PacketType::Auth => 0xF0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Copy)]
pub enum PacketType {
    Reserved,
    Connect,
    Connack,
    Subscribe,
    Suback,
    Publish,
    Puback,
    Pubrec,
    Pubrel,
    Pubcomp,
    Unsubscribe,
    Unsuback,
    Pingreq,
    Pingresp,
    Disconnect,
    Auth,
}

/// FixedHeader represents data in the first byte
/// that is always present, but not all flags are relevant
/// for every request
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FixedHeader {
    pub cmd: PacketType,
    pub dup: bool,
    pub qos: u8,
    pub retain: bool,
}

impl FixedHeader {
    pub fn from_byte(byte: u8) -> Res<FixedHeader> {
        let (cmd, flags) = (PacketType::from_bits(byte >> 4), byte & 0b00001111);
        // validate flags of header
        // TODO: validate later in publish when version != 5
        match (cmd, flags) {
            // Pubrel/Subscribe/Unsubscribe is always QoS 1
            (PacketType::Pubrel, 0) | (PacketType::Subscribe, 0) | (PacketType::Unsubscribe, 0) => {
                return Err(format!(
                    "Invalid header flag bits, must be 0x2 for {:?} packet",
                    cmd
                ))
            }
            // this should pass
            (PacketType::Publish, _)
            | (PacketType::Pubrel, 2)
            | (PacketType::Subscribe, 2)
            | (PacketType::Unsubscribe, 2)
            | (_, 0) => {}
            (t, f) => return Err(format!("Flags {:?} should not be set for type {:?}", f, t)),
        }
        let (retain, qos, dup) = (
            (flags & RETAIN_MASK) != 0,
            (flags & QOS_MASK) >> QOS_SHIFT,
            (flags & DUP_MASK) != 0,
        );
        if qos > 2 {
            return Err("Packet must not have both QoS bits set to 1".to_string());
        }
        Ok(FixedHeader {
            cmd,
            dup,
            retain,
            qos,
        })
    }

    pub fn for_type(cmd: PacketType) -> FixedHeader {
        FixedHeader {
            cmd,
            dup: false,
            qos: 0,
            retain: false,
        }
    }

    pub fn encode(&self) -> u8 {
        let message_type = PacketType::to_bits(self.cmd);
        match self.cmd {
            PacketType::Unsuback | PacketType::Unsubscribe => {
                message_type | ((self.dup as u8) << 3) | (self.qos << 1)
            }
            PacketType::Publish => {
                message_type | ((self.dup as u8) << 3) | (self.qos << 1) | self.retain as u8
            }
            PacketType::Subscribe => {
                message_type | 2 // Bits 3,2,1 and 0 need to ALWAYS be set to 0, 0, 1, 0 respectively
            }
            _ => message_type,
        }
    }
}
