use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type UserProperties = HashMap<String, Vec<String>>;
pub type Res<T> = Result<T, String>;

#[derive(PartialEq, Debug, Clone)]
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
    U32Vec(Vec<u32>),
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

#[derive(PartialEq, Debug)]
pub enum QoS {
    QoS0,
    QoS1,
    QoS2,
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

pub(crate) trait Properties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>>;
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
        // validate flags of header
        // TODO: validate later in publish when version != 5
        match (cmd, flags) {
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
            (flags >> QOS_SHIFT) & QOS_MASK,
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
        PacketType::to_bits(self.cmd)
    }
}

#[derive(Debug, PartialEq)]
pub struct AuthProperties {
    pub authentication_method: Option<String>,
    pub authentication_data: Option<String>,
    pub reason_string: Option<String>,
    pub user_properties: UserProperties,
}

impl AuthProperties {
    pub fn from_properties(props: Vec<(u8, PropType)>) -> Res<AuthProperties> {
        let mut reason_string = None;
        let mut user_properties = UserProperties::new();
        let mut authentication_method = None;
        let mut authentication_data = None;
        for p in props {
            match p {
                (0x1F, PropType::String(v)) => reason_string = Some(v),
                (0x26, PropType::Map(v)) => user_properties = v,
                (0x15, PropType::String(v)) => authentication_method = Some(v),
                (0x16, PropType::String(v)) => authentication_data = Some(v),
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(AuthProperties {
            reason_string,
            user_properties,
            authentication_method,
            authentication_data,
        })
    }
}

impl Properties for AuthProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if let Some(s) = self.reason_string {
            out.push((0x1F, PropType::String(s)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties)));
        }
        if let Some(s) = self.authentication_method {
            out.push((0x15, PropType::String(s)));
        }
        if let Some(s) = self.authentication_data {
            out.push((0x16, PropType::String(s)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq)]
pub struct AuthPacket {
    pub fixed: FixedHeader,
    pub reason_code: u8,
    pub properties: Option<AuthProperties>,
}

#[derive(Debug, PartialEq)]
pub struct PublishProperties {
    pub payload_format_indicator: bool,
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<String>,
    // Can be multiple identifiers
    pub subscription_identifier: Vec<u32>,
    // topic alias is None if absent
    pub topic_alias: Option<u16>,
    pub user_properties: UserProperties,
}

impl PublishProperties {
    pub fn from_properties(props: Vec<(u8, PropType)>) -> Res<PublishProperties> {
        let mut user_properties = UserProperties::new();
        let mut payload_format_indicator = false;
        let mut message_expiry_interval = None;
        let mut content_type = None;
        let mut response_topic = None;
        let mut correlation_data = None;
        let mut subscription_identifier = vec![];
        let mut topic_alias = None;
        for p in props {
            match p {
                (0x26, PropType::Map(v)) => user_properties = v,
                (0x01, PropType::Bool(v)) => payload_format_indicator = v,
                (0x02, PropType::U32(v)) => message_expiry_interval = Some(v),
                (0x03, PropType::String(v)) => content_type = Some(v),
                (0x08, PropType::String(v)) => response_topic = Some(v),
                (0x09, PropType::String(v)) => correlation_data = Some(v),
                (0x0B, PropType::U32Vec(v)) => subscription_identifier = v,
                (0x23, PropType::U16(v)) => topic_alias = Some(v),
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(PublishProperties {
            subscription_identifier,
            user_properties,
            payload_format_indicator,
            message_expiry_interval,
            content_type,
            response_topic,
            correlation_data,
            topic_alias,
        })
    }
}

impl Properties for PublishProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties)));
        }
        if let v = self.payload_format_indicator {
            out.push((0x01, PropType::Bool(v)));
        }
        if let Some(v) = self.message_expiry_interval {
            out.push((0x02, PropType::U32(v)));
        }
        if let Some(v) = self.content_type {
            out.push((0x03, PropType::String(v)));
        }
        if let Some(v) = self.response_topic {
            out.push((0x08, PropType::String(v)));
        }
        if let Some(v) = self.correlation_data {
            out.push((0x09, PropType::String(v)));
        }
        if let v = self.subscription_identifier {
            out.push((0x0B, PropType::U32Vec(v)));
        }
        if let Some(v) = self.topic_alias {
            out.push((0x23, PropType::U16(v)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq)]
pub struct SubscribeProperties {
    pub subscription_identifier: Vec<u32>,
    pub user_properties: UserProperties,
}

impl SubscribeProperties {
    pub fn from_properties(props: Vec<(u8, PropType)>) -> Res<SubscribeProperties> {
        let mut subscription_identifier = vec![];
        let mut user_properties = UserProperties::new();
        for p in props {
            match p {
                (0x0B, PropType::U32Vec(v)) => subscription_identifier = v,
                (0x26, PropType::Map(v)) => user_properties = v,
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(SubscribeProperties {
            subscription_identifier,
            user_properties,
        })
    }
}

impl Properties for SubscribeProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if let s = self.subscription_identifier {
            out.push((0x0B, PropType::U32Vec(s)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq)]
pub struct DisconnectProperties {
    pub session_expiry_interval: Option<u32>,
    pub server_reference: Option<String>,
    pub reason_string: Option<String>,
    pub user_properties: UserProperties,
}

impl DisconnectProperties {
    pub fn from_properties(prop_list: Vec<(u8, PropType)>) -> Res<DisconnectProperties> {
        let mut props = DisconnectProperties {
            session_expiry_interval: None,
            server_reference: None,
            reason_string: None,
            user_properties: UserProperties::new(),
        };
        for p in prop_list {
            match p {
                (0x11, PropType::U32(v)) => props.session_expiry_interval = Some(v),
                (0x26, PropType::Map(v)) => props.user_properties = v,
                (0x1C, PropType::String(v)) => props.server_reference = Some(v),
                (0x1F, PropType::String(v)) => props.reason_string = Some(v),
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(props)
    }
}

impl Properties for DisconnectProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if let Some(s) = self.session_expiry_interval {
            out.push((0x11, PropType::U32(s)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties)));
        }
        if let Some(v) = self.server_reference {
            out.push((0x1C, PropType::String(v)));
        }
        if let Some(v) = self.reason_string {
            out.push((0x1F, PropType::String(v)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConfirmationProperties {
    pub reason_string: Option<String>,
    pub user_properties: UserProperties,
}

#[derive(Debug, PartialEq)]
pub struct UnsubscribeProperties {
    pub user_properties: UserProperties,
}

impl UnsubscribeProperties {
    pub fn from_properties(props: Vec<(u8, PropType)>) -> Res<UnsubscribeProperties> {
        let mut user_properties = UserProperties::new();
        for p in props {
            match p {
                (0x26, PropType::Map(v)) => user_properties = v,
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(UnsubscribeProperties { user_properties })
    }
}

impl Properties for UnsubscribeProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties)));
        }
        Ok(out)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct LastWill {
    pub topic: Option<String>,
    pub payload: Option<String>,
    pub qos: u8,
    pub retain: bool,
    pub properties: WillProperties,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ConnectProperties {
    // defaults to 0
    pub session_expiry_interval: u32,
    // defaults to 65,535
    pub receive_maximum: u16,
    // if None then no limit
    pub maximum_packet_size: Option<u32>,
    // default value is 0
    pub topic_alias_maximum: u16,
    // default is false
    pub request_response_information: bool,
    // default is true
    pub request_problem_information: bool,
    // default is just an empty hashMap
    pub user_properties: UserProperties,
    // default is None
    pub authentication_method: Option<String>,
    pub authentication_data: Option<String>,
}

impl Default for ConnectProperties {
    fn default() -> ConnectProperties {
        ConnectProperties {
            session_expiry_interval: 0,
            receive_maximum: 0xffff,
            maximum_packet_size: None,
            topic_alias_maximum: 0,
            request_response_information: false,
            request_problem_information: true,
            user_properties: UserProperties::new(),
            // extended_auth: None,
            authentication_method: None,
            authentication_data: None,
        }
    }
}

impl ConnectProperties {
    pub fn from_properties(props: Vec<(u8, PropType)>) -> Res<ConnectProperties> {
        let mut out = ConnectProperties::default();
        for p in props {
            match p {
                (0x11, PropType::U32(v)) => out.session_expiry_interval = v,
                (0x15, PropType::String(v)) => out.authentication_method = Some(v),
                (0x16, PropType::String(v)) => out.authentication_data = Some(v),
                (0x17, PropType::Bool(v)) => out.request_problem_information = v,
                (0x19, PropType::Bool(v)) => out.request_response_information = v,
                (0x21, PropType::U16(v)) => out.receive_maximum = v,
                (0x22, PropType::U16(v)) => out.topic_alias_maximum = v,
                (0x26, PropType::Map(v)) => out.user_properties = v,
                (0x27, PropType::U32(v)) => out.maximum_packet_size = Some(v),
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(out)
    }
}

impl Properties for ConnectProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if let v = self.session_expiry_interval {
            out.push((0x11, PropType::U32(v)));
        }
        if let Some(v) = self.authentication_method {
            out.push((0x15, PropType::String(v)));
        }
        if let Some(v) = self.authentication_data {
            out.push((0x16, PropType::String(v)));
        }
        if let v = self.request_problem_information {
            out.push((0x17, PropType::Bool(v)));
        }
        if let v = self.request_response_information {
            out.push((0x19, PropType::Bool(v)));
        }
        if let v = self.receive_maximum {
            out.push((0x21, PropType::U16(v)));
        }
        if let v = self.topic_alias_maximum {
            out.push((0x22, PropType::U16(v)));
        }
        if let v = self.user_properties {
            out.push((0x26, PropType::Map(v)));
        }
        if let Some(v) = self.maximum_packet_size {
            out.push((0x27, PropType::U32(v)));
        }
        Ok(out)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct WillProperties {
    /// default value is false, both if set to false
    /// and when no value was provided
    pub payload_format_indicator: bool,
    /// None if no value was given, because
    /// apparently 0 is a valid expiry
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<String>,
    /// 0 is default when no value was provided
    pub will_delay_interval: u32,
    pub user_properties: UserProperties,
}

impl Default for WillProperties {
    fn default() -> WillProperties {
        WillProperties {
            payload_format_indicator: false,
            message_expiry_interval: None,
            content_type: None,
            response_topic: None,
            correlation_data: None,
            will_delay_interval: 0,
            user_properties: UserProperties::new(),
        }
    }
}

impl WillProperties {
    pub fn from_properties(props: Vec<(u8, PropType)>) -> Res<WillProperties> {
        let mut out = WillProperties::default();
        for p in props {
            match p {
                (0x01, PropType::Bool(v)) => out.payload_format_indicator = v,
                (0x02, PropType::U32(v)) => out.message_expiry_interval = Some(v),
                (0x03, PropType::String(v)) => out.content_type = Some(v),
                (0x08, PropType::String(v)) => out.response_topic = Some(v),
                (0x09, PropType::String(v)) => out.correlation_data = Some(v),
                (0x18, PropType::U32(v)) => out.will_delay_interval = v,
                (0x26, PropType::Map(v)) => out.user_properties = v,
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(out)
    }
}

impl Properties for WillProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties)));
        }
        if let v = self.payload_format_indicator {
            out.push((0x01, PropType::Bool(v)));
        }
        if let Some(v) = self.message_expiry_interval {
            out.push((0x02, PropType::U32(v)));
        }
        if let Some(v) = self.content_type {
            out.push((0x03, PropType::String(v)));
        }
        if let Some(v) = self.response_topic {
            out.push((0x08, PropType::String(v)));
        }
        if let Some(v) = self.correlation_data {
            out.push((0x09, PropType::String(v)));
        }
        if let v = self.will_delay_interval {
            out.push((0x18, PropType::U32(v)));
        }
        Ok(out)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ConnectPacket {
    pub fixed: FixedHeader,
    pub length: u32,
    pub client_id: String,
    pub protocol_version: u8,
    pub protocol_id: Protocol,
    pub clean_session: bool,
    pub keep_alive: u16,
    pub user_name: Option<String>,
    pub password: Option<String>,
    /// a last will is not mandatory
    pub will: Option<LastWill>,
    pub properties: Option<ConnectProperties>,
}

#[derive(Debug, PartialEq)]
pub struct ConnackPacket {
    pub fixed: FixedHeader,
    pub length: u32,
    pub return_code: Option<u8>,
    pub reason_code: Option<u8>,
    pub session_present: bool,
    pub properties: Option<ConnackProperties>,
}

impl Default for ConnackPacket {
    fn default() -> ConnackPacket {
        ConnackPacket {
            fixed: FixedHeader::for_type(PacketType::Connack),
            length: 0,
            return_code: None,
            reason_code: None,
            session_present: false,
            properties: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ConnackProperties {
    pub session_expiry_interval: u32,
    pub assigned_client_identifier: Option<String>,
    pub server_keep_alive: Option<u16>,
    pub authentication_method: Option<String>,
    pub authentication_data: Option<String>,
    pub response_information: Option<String>,
    pub server_reference: Option<String>,
    pub reason_string: Option<String>,
    pub receive_maximum: u16,
    pub topic_alias_maximum: u16,
    pub maximum_qos: u8,
    pub retain_available: bool,
    pub user_properties: UserProperties,
    pub maximum_packet_size: Option<u32>,
    pub wildcard_subscription_available: bool,
    pub subscription_identifiers_available: bool,
    pub shared_subscription_available: bool,
    pub is_default: bool,
}

impl Default for ConnackProperties {
    fn default() -> ConnackProperties {
        ConnackProperties {
            is_default: true,
            session_expiry_interval: 0,
            receive_maximum: 0xffff,
            maximum_packet_size: None,
            topic_alias_maximum: 0,
            response_information: None,
            // request_problem_information: true,
            user_properties: UserProperties::new(),
            assigned_client_identifier: None,
            server_keep_alive: None,
            authentication_method: None,
            authentication_data: None,
            server_reference: None,
            reason_string: None,
            maximum_qos: 2,
            // retained messages available by default
            retain_available: true,
            wildcard_subscription_available: false,
            subscription_identifiers_available: false,
            shared_subscription_available: false,
        }
    }
}

impl ConnackProperties {
    pub fn from_properties(props: Vec<(u8, PropType)>) -> Res<ConnackProperties> {
        let mut out = ConnackProperties::default();
        for p in props {
            out.is_default = false;
            match p {
                (0x11, PropType::U32(v)) => out.session_expiry_interval = v,
                (0x12, PropType::String(v)) => out.assigned_client_identifier = Some(v),
                (0x13, PropType::U16(v)) => out.server_keep_alive = Some(v),
                (0x15, PropType::String(v)) => out.authentication_method = Some(v),
                (0x16, PropType::String(v)) => out.authentication_data = Some(v),
                (0x1A, PropType::String(v)) => out.response_information = Some(v),
                (0x1C, PropType::String(v)) => out.server_reference = Some(v),
                (0x1F, PropType::String(v)) => out.reason_string = Some(v),
                (0x21, PropType::U16(v)) => out.receive_maximum = v,
                (0x22, PropType::U16(v)) => out.topic_alias_maximum = v,
                (0x24, PropType::U8(v)) => out.maximum_qos = v,
                (0x25, PropType::Bool(v)) => out.retain_available = v,
                (0x26, PropType::Map(v)) => out.user_properties = v,
                (0x27, PropType::U32(v)) => out.maximum_packet_size = Some(v),
                (0x28, PropType::Bool(v)) => out.wildcard_subscription_available = v,
                (0x29, PropType::Bool(v)) => out.subscription_identifiers_available = v,
                (0x2A, PropType::Bool(v)) => out.shared_subscription_available = v,
                v => return Err(format!("Failed to get connack properties {:?}", v)),
            }
        }
        Ok(out)
    }
}

impl Properties for ConnackProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties)));
        }
        // TODO remove irrefutable patterns, might need to change structure
        if let v = self.session_expiry_interval {
            out.push((0x11, PropType::U32(v)));
        }
        if let Some(v) = self.assigned_client_identifier {
            out.push((0x12, PropType::String(v)));
        }
        if let Some(v) = self.server_keep_alive {
            out.push((0x13, PropType::U16(v)));
        }
        if let Some(v) = self.authentication_method {
            out.push((0x15, PropType::String(v)));
        }
        if let Some(v) = self.authentication_data {
            out.push((0x16, PropType::String(v)));
        }
        if let Some(v) = self.response_information {
            out.push((0x1A, PropType::String(v)));
        }
        if let Some(v) = self.server_reference {
            out.push((0x1C, PropType::String(v)));
        }
        if let Some(v) = self.reason_string {
            out.push((0x1F, PropType::String(v)));
        }
        if let v = self.receive_maximum {
            out.push((0x21, PropType::U16(v)));
        }
        if let v = self.topic_alias_maximum {
            out.push((0x22, PropType::U16(v)));
        }
        if let v = self.maximum_qos {
            out.push((0x24, PropType::U8(v)));
        }
        if let v = self.retain_available {
            out.push((0x25, PropType::Bool(v)));
        }
        if let Some(v) = self.maximum_packet_size {
            out.push((0x27, PropType::U32(v)));
        }
        if let v = self.wildcard_subscription_available {
            out.push((0x28, PropType::Bool(v)));
        }
        if let v = self.subscription_identifiers_available {
            out.push((0x29, PropType::Bool(v)));
        }
        if let v = self.shared_subscription_available {
            out.push((0x2A, PropType::Bool(v)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Clone)]
/// A struct for PUBACK, PUBCOMP, PUBREL and PUBREC
pub struct ConfirmationPacket {
    pub fixed: FixedHeader,
    pub length: u32,
    /// Reason code is always 0 by default for MQTT 5
    /// but absent for MQTT 3 and 4
    pub reason_code: Option<u8>,
    pub properties: Option<ConfirmationProperties>,
    pub message_id: u16,
}

impl ConfirmationProperties {
    pub fn from_properties(props: Vec<(u8, PropType)>) -> Res<ConfirmationProperties> {
        let mut reason_string = None;
        let mut user_properties = UserProperties::new();
        for p in props {
            match p {
                (0x1F, PropType::String(v)) => reason_string = Some(v),
                (0x26, PropType::Map(v)) => user_properties = v,
                s => return Err(format!("Failed to parse connect properties {:?}", s)),
            }
        }
        Ok(ConfirmationProperties {
            reason_string,
            user_properties,
        })
    }
}

impl Properties for ConfirmationProperties {
    fn to_pairs(self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if let Some(s) = self.reason_string {
            out.push((0x1F, PropType::String(s)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq)]
pub struct DisconnectPacket {
    pub fixed: FixedHeader,
    // only exists in MQTT 5
    pub reason_code: Option<u8>,
    pub properties: Option<DisconnectProperties>,
}

#[derive(Debug, PartialEq)]
pub struct ReasonCode {}

impl ReasonCode {
    pub fn validate_puback_pubrec_code(code: u8) -> Res<u8> {
        match code {
            0x00 // 'Success',
            | 0x10 // 'No matching subscribers',
            | 0x80 // 'Unspecified error',
            | 0x83 // 'Implementation specific error',
            | 0x87 // 'Not authorized',
            | 0x90 // 'Topic Name invalid',
            | 0x91 // 'Packet identifier in use',
            | 0x97 // 'Quota exceeded',
            | 0x99  // 'Payload format invalid'
            => Ok(code),
            _ => Err(format!("Invalid puback/pubrec code {}", code))
        }
    }

    pub fn validate_pubcomp_pubrel_code(code: u8) -> Res<u8> {
        match code {
            0x00  // 'Success',
            | 0x92 => Ok(code),  // 'Packet Identifier not found',
            _ => Err(format!("Invalid pubcomp/pubrel code {}", code))
        }
    }

    pub fn validate_suback_code(code: u8) -> Res<u8> {
        match code {
            0x00 | // 'Granted QoS 0',
            0x01 | // 'Granted QoS 1',
            0x02 | // 'Granted QoS 2',
            0x80 | // 'Unspecified error',
            0x83 | // 'Implementation specific error',
            0x87 | // 'Not authorized',
            0x8F | // 'Topic Filter invalid',
            0x91 | // 'Packet Identifier in use',
            0x97 | // 'Quota exceeded',
            0x9E | // 'Shared Subscriptions not supported',
            0xA1 | // 'Subscription Identifiers not supported',
            0xA2 => Ok(code),// 'Wildcard Subscriptions not supported',
            _ => Err(format!("Invalid suback code {}", code))
        }
    }

    pub fn validate_unsuback_code(code: u8) -> Res<u8> {
        match code {
            0x00 // 'Success',
            | 0x11 // 'No subscription existed',
            | 0x80 // 'Unspecified error',
            | 0x83 // 'Implementation specific error',
            | 0x87 // 'Not authorized',
            | 0x8F // 'Topic Filter invalid',
            | 0x91 =>Ok(code), // 'Packet Identifier in use'
            _ => Err(format!("Invalid unsuback code {}", code)),
        }
    }

    pub fn validate_disconnect_code(code: u8) -> Res<u8> {
        match code {
            0x00 // 'Normal disconnection',
            | 0x04 // 'Disconnect with Will Message',
            | 0x80 // 'Unspecified error',
            | 0x81 // 'Malformed Packet',
            | 0x82 // 'Protocol Error',
            | 0x83 // 'Implementation specific error',
            | 0x87 // 'Not authorized',
            | 0x89 // 'Server busy',
            | 0x8B // 'Server shutting down',
            | 0x8D // 'Keep Alive timeout',
            | 0x8E // 'Session taken over',
            | 0x8F // 'Topic Filter invalid',
            | 0x90 // 'Topic Name invalid',
            | 0x93 // 'Receive Maximum exceeded',
            | 0x94 // 'Topic Alias invalid',
            | 0x95 // 'Packet too large',
            | 0x96 // 'Message rate too high',
            | 0x97 // 'Quota exceeded',
            | 0x98 // 'Administrative action',
            | 0x99 // 'Payload format invalid',
            | 0x9A // 'Retain not supported',
            | 0x9B // 'QoS not supported',
            | 0x9C // 'Use another server',
            | 0x9D // 'Server moved',
            | 0x9E // 'Shared Subscriptions not supported',
            | 0x9F // 'Connection rate exceeded',
            | 0xA0 // 'Maximum connect time',
            | 0xA1 // 'Subscription Identifiers not supported',
            | 0xA2 => Ok(code), // 'Wildcard Subscriptions not supported'
            _ => Err(format!("Invalid disconnect code {}", code)),
        }
    }

    pub fn validate_auth_code(code: u8) -> Res<u8> {
        match code {
            0x00 // 'Success',
            | 0x18 // 'Continue authentication',
            | 0x19 => Ok(code), // 'Re-authenticate'
            _ => Err(format!("Invalid auth code {}", code)),
        }
    }
}

#[derive(Debug, PartialEq)]
/// Captures value of published message
pub struct PublishPacket {
    pub fixed: FixedHeader,
    pub topic: String,
    pub message_id: Option<u16>,
    /// No assumptions are made about the structure
    /// and content of payload
    pub payload: Vec<u8>,
    /// Used in MQTT 5
    pub properties: Option<PublishProperties>,
}

#[derive(Debug, PartialEq)]
pub struct Subscription {
    pub topic: String,
    pub qos: QoS,
    pub nl: bool,
    pub rap: bool,
    pub rh: Option<u8>,
}

#[derive(Debug, PartialEq)]
pub struct SubscribePacket {
    pub fixed: FixedHeader,
    pub subscriptions: Vec<Subscription>,
    pub properties: Option<SubscribeProperties>,
    pub message_id: u16,
}

#[derive(Debug, PartialEq)]
/// Use an enum to make setting the reason code easier and safer
pub enum GrantedReasonCode {
    /// 0x00  The subscription is accepted and the maximum QoS sent will be QoS 0. This might be a lower QoS than was requested.
    GrantedQoS0,
    /// 0x01  The subscription is accepted and the maximum QoS sent will be QoS 1. This might be a lower QoS than was requested.
    GrantedQoS1,
    /// 0x02  The subscription is accepted and any received QoS will be sent to this subscription.
    GrantedQoS2,
    /// 0x80 The subscription is not accepted and the Server either does not wish to reveal the reason or none of the other Reason Codes apply.
    UnspecifiedError,
    /// 0x83 The SUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError,
    /// 0x87  The Client is not authorized to make this subscription.
    NotAuthorized,
    /// 0x8F The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInvalid,
    /// 0x91 The specified Packet Identifier is already in use.
    PacketIdentifierInUse,
    /// 0x97  An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded,
    /// 0x9E The Server does not support Shared Subscriptions for this Client.
    SharedSubscriptionsNot,
    /// 0xA1 The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifiersNotSupported,
    /// 0xA2 The Server does not support Wildcard Subscriptions; the subscription is not accepted.
    WildcardSubscriptionsNotSupported,
}

impl GrantedReasonCode {
    pub fn from_byte(byte: u8) -> Res<GrantedReasonCode> {
        match byte {
            0x00 => Ok(GrantedReasonCode::GrantedQoS0),
            0x01 => Ok(GrantedReasonCode::GrantedQoS1),
            0x02 => Ok(GrantedReasonCode::GrantedQoS2),
            0x80 => Ok(GrantedReasonCode::UnspecifiedError),
            0x83 => Ok(GrantedReasonCode::ImplementationSpecificError),
            0x87 => Ok(GrantedReasonCode::NotAuthorized),
            0x8F => Ok(GrantedReasonCode::TopicFilterInvalid),
            0x91 => Ok(GrantedReasonCode::PacketIdentifierInUse),
            0x97 => Ok(GrantedReasonCode::QuotaExceeded),
            0x9E => Ok(GrantedReasonCode::SharedSubscriptionsNot),
            0xA1 => Ok(GrantedReasonCode::SubscriptionIdentifiersNotSupported),
            0xA2 => Ok(GrantedReasonCode::WildcardSubscriptionsNotSupported),
            // fallback to unspecified error to keep function signature simple
            _ => Err(format!("Invalid granted reason code {}", byte)),
        }
    }

    pub fn to_byte(self: GrantedReasonCode) -> u8 {
        match self {
            GrantedReasonCode::GrantedQoS0 => 0x00,
            GrantedReasonCode::GrantedQoS1 => 0x01,
            GrantedReasonCode::GrantedQoS2 => 0x02,
            GrantedReasonCode::UnspecifiedError => 0x80,
            GrantedReasonCode::ImplementationSpecificError => 0x83,
            GrantedReasonCode::NotAuthorized => 0x87,
            GrantedReasonCode::TopicFilterInvalid => 0x8F,
            GrantedReasonCode::PacketIdentifierInUse => 0x91,
            GrantedReasonCode::QuotaExceeded => 0x97,
            GrantedReasonCode::SharedSubscriptionsNot => 0x9E,
            GrantedReasonCode::SubscriptionIdentifiersNotSupported => 0xA1,
            GrantedReasonCode::WildcardSubscriptionsNotSupported => 0xA2,
        }
    }
}

#[derive(Debug, PartialEq)]
/// Packet that holds information of subscription acknowledgement (SUBACK)
pub struct SubackPacket {
    pub fixed: FixedHeader,
    pub reason_code: Option<u8>,
    pub properties: Option<ConfirmationProperties>,
    /// used in MQTT 5
    pub granted_reason_codes: Vec<GrantedReasonCode>,
    /// used in MQTT 3.1 and 4
    pub granted_qos: Vec<QoS>,
}

#[derive(Debug, PartialEq)]
pub struct UnsubscribePacket {
    pub fixed: FixedHeader,
    pub properties: Option<UnsubscribeProperties>,
    pub unsubscriptions: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct UnsubackPacket {
    pub fixed: FixedHeader,
    /// used only in MQTT 5, will always empty if
    /// not MQTT 5
    pub granted: Vec<u8>,
    pub properties: Option<ConfirmationProperties>,
}

#[derive(Debug, PartialEq)]
pub struct PingreqPacket {
    pub fixed: FixedHeader,
}

#[derive(Debug, PartialEq)]
pub struct PingrespPacket {
    pub fixed: FixedHeader,
}
