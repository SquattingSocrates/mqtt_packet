use crate::connect::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, Read};

pub type UserProperties = HashMap<String, Vec<String>>;
pub type Res<T> = Result<T, String>;

#[derive(PartialEq, Debug)]
pub enum Protocol {
    Mqtt,
    MQIsdp,
}

#[derive(Debug)]
pub enum PropType {
    U32(u32),
    U16(u16),
    U8(u8),
    String(String),
    Pair(String, String),
    Map(UserProperties),
    Bool(bool),
}

impl Protocol {
    pub fn from_str(s: &str) -> Protocol {
        match s {
            "MQIsdp" => Protocol::MQIsdp,
            "MQTT" => Protocol::Mqtt,
            // TODO: return a result
            _ => Protocol::Mqtt,
        }
    }
}

#[derive(PartialEq)]
pub enum QoS {
    QoS0,
    QoS1,
    QoS2,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

static DUP_MASK: u8 = 0x8;
static QOS_MASK: u8 = 0x6;
static QOS_SHIFT: u8 = 1;
static RETAIN_MASK: u8 = 0x1;

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

    /// returns flags that need to be set for certain message types
    fn requires_header_flags(flags: u8) -> u8 {
        match flags {
            // Pubrel, subscribe and unsubscribe
            6 | 8 | 10 => 2,
            _ => 0,
        }
    }
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
    pub fn from_byte(byte: u8) -> Result<FixedHeader, String> {
        let (cmd, flags) = (PacketType::from_bits(byte >> 4), byte & 0b00001111);
        if let 2 = PacketType::requires_header_flags(flags) {
            if flags & 0xf != 2 {
                return Err("Invalid flags set".to_string());
            }
        }
        let (retain, qos, dup) = (
            (flags & RETAIN_MASK) != 0,
            (flags >> QOS_SHIFT) & QOS_MASK,
            (flags & DUP_MASK) != 0,
        );
        if qos > 2 {
            return Err("Packet must not have both QoS bits set to 1".to_string());
        }
        Ok(FixedHeader {
            cmd,
            dup: flags >= 8,
            qos: (flags & 0x6) >> 1,
            retain: flags & 0x1 > 0,
        })
    }
}

pub struct AuthProperties {
    authentication_method: Option<String>,
    authentication_data: Option<String>,
    reason_string: Option<String>,
    user_properties: UserProperties,
}

pub struct AuthPacket {
    fixed: FixedHeader,
    message_id: Option<u32>,
    reason_code: u8,
    properties: Option<AuthProperties>,
}

pub struct PublishProperties {
    payload_format_indicator: u8,
    message_expiry_interval: u32,
    content_type: Option<String>,
    response_topic: Option<String>,
    correlation_data: Option<String>,
    subscription_identifier: u32,
    topic_alias: u16,
    user_property: UserProperties,
}

pub struct SubscribeProperties {
    subscription_identifier: u32,
    user_property: UserProperties,
}

pub struct ConnackProperties {
    session_expiry_interval: u32,
    assigned_client_identifier: Option<String>,
    server_keep_alive: u16,
    authentication_method: Option<String>,
    authentication_data: Option<String>,
    response_information: Option<String>,
    server_reference: Option<String>,
    reason_string: Option<String>,
    receive_maximum: u16,
    topic_alias_maximum: u16,
    maximum_qoS: u8,
    retain_available: u8,
    user_property: UserProperties,
    maximum_packet_size: u32,
    wildcard_subscription_available: u8,
    subscription_identifier_available: u8,
    shared_subscription_available: u8,
}

pub struct DisconnectProperties {
    session_expiry_interval: u32,
    server_reference: Option<String>,
    reason_string: Option<String>,
    user_property: UserProperties,
}

pub struct PubackProperties {
    reason_string: Option<String>,
    user_property: UserProperties,
}

pub struct PubrecProperties {
    reason_string: Option<String>,
    user_property: UserProperties,
}

pub struct PubrelProperties {
    reason_string: Option<String>,
    user_property: UserProperties,
}

pub struct PubcompProperties {
    reason_string: Option<String>,
    user_property: UserProperties,
}

pub struct SubackProperties {
    reason_string: Option<String>,
    user_property: UserProperties,
}

pub struct UnsubackProperties {
    reason_string: Option<String>,
    user_property: UserProperties,
}

pub struct UnsubscribeProperties {
    user_property: UserProperties,
}
