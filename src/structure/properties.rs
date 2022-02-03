use super::codes::*;
use super::common::*;
use crate::mqtt_writer::MqttWriter;
use serde::{Deserialize, Serialize};

/// Turn any particular type of PropertiesObject
/// to list of code - Value pairs
pub(crate) trait Properties: Sized {
    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>>;
    fn from_properties(props: Vec<(u8, PropType)>) -> Res<Self>;

    /// packet.encode() will move the value since a packet is usually built to
    /// be encoded and sent anyway
    fn encode(&self) -> Res<Vec<u8>> {
        // Confirm should not add empty property length with no properties (rfc 3.4.2.2.1)
        let pairs = self.to_pairs()?;
        if pairs.is_empty() {
            return Ok(vec![0]); // empty properties
        }
        // TODO: calculate size of properties before allocating buffer
        let mut writer = MqttWriter::new(100);
        writer.write_properties(pairs)?;
        Ok(writer.into_vec())
    }

    fn encode_option(props: Option<&Self>, protocol_version: u8) -> Res<(Vec<u8>, Vec<u8>)> {
        // Confirm should not add empty property length with no properties (rfc 3.4.2.2.1)
        if protocol_version == 5 {
            if let Some(p) = props {
                let enc = p.encode()?;
                Ok((MqttWriter::encode_variable_num(enc.len() as u32), enc))
            } else {
                Ok((vec![0], vec![])) // empty properties
            }
        } else {
            Ok((vec![], vec![])) // no properties exist in MQTT < 5
        }
    }
}

#[derive(Debug)]
pub enum PropType<'a> {
    U32(u32),
    U16(u16),
    U8(u8),
    Str(&'a str),
    String(String),
    Pair(String, String),
    Map(UserProperties),
    /// since when decoding we need to move the map
    /// it's better to keep two separate variants for
    /// refs and values
    MapRef(&'a UserProperties),
    Bool(bool),
    U32Vec(Vec<u32>),
    Binary(Vec<u8>),
    BinaryRef(&'a [u8]),
    VarInt(u32),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AuthProperties {
    pub authentication_method: String,
    pub authentication_data: Option<String>,
    pub reason_string: Option<String>,
    pub user_properties: UserProperties,
}

impl Properties for AuthProperties {
    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![(0x15, PropType::Str(&self.authentication_method))];
        if let Some(s) = self.authentication_data.as_ref() {
            out.push((0x16, PropType::Str(s)));
        }
        if let Some(s) = self.reason_string.as_ref() {
            out.push((0x1F, PropType::Str(s)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties.clone())));
        }
        Ok(out)
    }

    fn from_properties(props: Vec<(u8, PropType)>) -> Res<AuthProperties> {
        let mut reason_string = None;
        let mut user_properties = UserProperties::new();
        let mut authentication_method = String::new();
        let mut authentication_data = None;
        for p in props {
            match p {
                (0x1F, PropType::String(v)) => reason_string = Some(v),
                (0x26, PropType::Map(v)) => user_properties = v,
                (0x15, PropType::String(v)) => authentication_method = v,
                (0x16, PropType::String(v)) => authentication_data = Some(v),
                s => return Err(format!("Failed to parse auth properties {:?}", s)),
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PublishProperties {
    pub payload_format_indicator: bool,
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Vec<u8>,
    // Can be multiple identifiers
    pub subscription_identifiers: Vec<u32>,
    // topic alias is None if absent
    pub topic_alias: Option<u16>,
    pub user_properties: UserProperties,
}

impl Properties for PublishProperties {
    fn from_properties(props: Vec<(u8, PropType)>) -> Res<PublishProperties> {
        let mut user_properties = UserProperties::new();
        let mut payload_format_indicator = false;
        let mut message_expiry_interval = None;
        let mut content_type = None;
        let mut response_topic = None;
        let mut correlation_data = vec![];
        let mut subscription_identifiers = vec![];
        let mut topic_alias = None;
        for p in props {
            match p {
                (0x26, PropType::Map(v)) => user_properties = v,
                (0x01, PropType::Bool(v)) => payload_format_indicator = v,
                (0x02, PropType::U32(v)) => message_expiry_interval = Some(v),
                (0x03, PropType::String(v)) => content_type = Some(v),
                (0x08, PropType::String(v)) => response_topic = Some(v),
                (0x09, PropType::Binary(v)) => correlation_data = v,
                (0x0B, PropType::VarInt(v)) => subscription_identifiers.push(v),
                (0x23, PropType::U16(v)) => topic_alias = Some(v),
                s => return Err(format!("Failed to parse publish properties {:?}", s)),
            }
        }
        Ok(PublishProperties {
            subscription_identifiers,
            user_properties,
            payload_format_indicator,
            message_expiry_interval,
            content_type,
            response_topic,
            correlation_data,
            topic_alias,
        })
    }

    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![(0x01, PropType::Bool(self.payload_format_indicator))];
        if let Some(v) = self.message_expiry_interval {
            out.push((0x02, PropType::U32(v)));
        }
        if let Some(v) = self.topic_alias {
            out.push((0x23, PropType::U16(v)));
        }
        if let Some(v) = self.response_topic.as_ref() {
            out.push((0x08, PropType::Str(v)));
        }
        if !self.correlation_data.is_empty() {
            out.push((0x09, PropType::BinaryRef(&self.correlation_data)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::MapRef(&self.user_properties)));
        }
        if !self.subscription_identifiers.is_empty() {
            for id in self.subscription_identifiers.iter() {
                if *id > VARBYTEINT_MAX {
                    return Err(format!("Invalid subscription_identifier: {}", id));
                }
                out.push((0x0B, PropType::VarInt(*id)));
            }
        }
        if let Some(v) = self.content_type.as_ref() {
            out.push((0x03, PropType::Str(v)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SubscribeProperties {
    /// subscription_identifiers is a variable length int
    /// and is not allowed to be 0. If it is set to 0 it won't
    /// encode/decode, therefore we're using 0 as None here
    pub subscription_identifier: u32,
    pub user_properties: UserProperties,
}

impl Properties for SubscribeProperties {
    fn from_properties(props: Vec<(u8, PropType)>) -> Res<SubscribeProperties> {
        let mut subscription_identifier = 0;
        let mut user_properties = UserProperties::new();
        for p in props {
            match p {
                (0x0B, PropType::VarInt(v)) => subscription_identifier = v,
                (0x26, PropType::Map(v)) => user_properties = v,
                s => return Err(format!("Failed to parse subscribe properties {:?}", s)),
            }
        }
        Ok(SubscribeProperties {
            subscription_identifier,
            user_properties,
        })
    }
    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if self.subscription_identifier > 0 {
            out.push((0x0B, PropType::VarInt(self.subscription_identifier)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::MapRef(&self.user_properties)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DisconnectProperties {
    pub session_expiry_interval: Option<u32>,
    pub server_reference: Option<String>,
    pub reason_string: Option<String>,
    pub user_properties: UserProperties,
}

impl Properties for DisconnectProperties {
    fn from_properties(prop_list: Vec<(u8, PropType)>) -> Res<DisconnectProperties> {
        let mut props = DisconnectProperties {
            session_expiry_interval: None,
            server_reference: None,
            reason_string: None,
            user_properties: UserProperties::new(),
        };
        for p in prop_list {
            match p {
                (0x11, PropType::U32(v)) => props.session_expiry_interval = Some(v),
                (0x1F, PropType::String(v)) => props.reason_string = Some(v),
                (0x26, PropType::Map(v)) => props.user_properties = v,
                (0x1C, PropType::String(v)) => props.server_reference = Some(v),
                s => return Err(format!("Failed to parse disconnect properties {:?}", s)),
            }
        }
        Ok(props)
    }

    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if let Some(s) = self.session_expiry_interval {
            out.push((0x11, PropType::U32(s)));
        }
        if let Some(v) = self.reason_string.as_ref() {
            out.push((0x1F, PropType::Str(v)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::MapRef(&self.user_properties)));
        }
        if let Some(v) = self.server_reference.as_ref() {
            out.push((0x1C, PropType::Str(v)));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfirmationProperties {
    pub reason_string: Option<String>,
    pub user_properties: UserProperties,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct UnsubscribeProperties {
    pub user_properties: UserProperties,
}

impl Properties for UnsubscribeProperties {
    fn from_properties(props: Vec<(u8, PropType)>) -> Res<UnsubscribeProperties> {
        let mut user_properties = UserProperties::new();
        for p in props {
            match p {
                (0x26, PropType::Map(v)) => user_properties = v,
                s => return Err(format!("Failed to parse unsubscribe properties {:?}", s)),
            }
        }
        Ok(UnsubscribeProperties { user_properties })
    }

    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::MapRef(&self.user_properties)));
        }
        Ok(out)
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Default)]
pub struct WillProperties {
    /// default value is false, both if set to false
    /// and when no value was provided
    pub payload_format_indicator: bool,
    /// None if no value was given, because
    /// apparently 0 is a valid expiry
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Vec<u8>,
    /// 0 is default when no value was provided
    pub will_delay_interval: u32,
    pub user_properties: UserProperties,
}

impl Properties for WillProperties {
    fn from_properties(props: Vec<(u8, PropType)>) -> Res<WillProperties> {
        let mut out = WillProperties::default();
        for p in props {
            match p {
                (0x01, PropType::Bool(v)) => out.payload_format_indicator = v,
                (0x02, PropType::U32(v)) => out.message_expiry_interval = Some(v),
                (0x03, PropType::String(v)) => out.content_type = Some(v),
                (0x08, PropType::String(v)) => out.response_topic = Some(v),
                (0x09, PropType::Binary(v)) => out.correlation_data = v,
                (0x18, PropType::U32(v)) => out.will_delay_interval = v,
                (0x26, PropType::Map(v)) => out.user_properties = v,
                s => return Err(format!("Failed to parse will properties {:?}", s)),
            }
        }
        Ok(out)
    }

    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        // if let v = self.will_delay_interval {
        out.push((0x18, PropType::U32(self.will_delay_interval)));
        // }
        // if let v = self.payload_format_indicator {
        out.push((0x01, PropType::Bool(self.payload_format_indicator)));
        // }
        if let Some(v) = self.message_expiry_interval {
            out.push((0x02, PropType::U32(v)));
        }
        if let Some(v) = self.content_type.as_ref() {
            out.push((0x03, PropType::Str(v)));
        }
        if let Some(v) = self.response_topic.as_ref() {
            out.push((0x08, PropType::Str(v)));
        }
        if !self.correlation_data.is_empty() {
            out.push((0x09, PropType::BinaryRef(&self.correlation_data)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties.clone())));
        }
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

impl Properties for ConnackProperties {
    fn from_properties(props: Vec<(u8, PropType)>) -> Res<ConnackProperties> {
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

    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::Map(self.user_properties.clone())));
        }
        // TODO remove irrefutable patterns, might need to change structure
        // if let v = self.session_expiry_interval {
        out.push((0x11, PropType::U32(self.session_expiry_interval)));
        // }
        if let Some(v) = self.assigned_client_identifier.as_ref() {
            out.push((0x12, PropType::Str(v)));
        }
        if let Some(v) = self.server_keep_alive {
            out.push((0x13, PropType::U16(v)));
        }
        if let Some(v) = self.authentication_method.as_ref() {
            out.push((0x15, PropType::Str(v)));
        }
        if let Some(v) = self.authentication_data.as_ref() {
            out.push((0x16, PropType::Str(v)));
        }
        if let Some(v) = self.response_information.as_ref() {
            out.push((0x1A, PropType::Str(v)));
        }
        if let Some(v) = self.server_reference.as_ref() {
            out.push((0x1C, PropType::Str(v)));
        }
        if let Some(v) = self.reason_string.as_ref() {
            out.push((0x1F, PropType::Str(v)));
        }
        // TODO: check all properties and maybe change types to Option<value>
        // if let v = self.receive_maximum {
        out.push((0x21, PropType::U16(self.receive_maximum)));
        // }
        // if let v = self.topic_alias_maximum {
        out.push((0x22, PropType::U16(self.topic_alias_maximum)));
        // }
        // if let v = self.maximum_qos {
        out.push((0x24, PropType::U8(self.maximum_qos)));
        // }
        // if let v = self.retain_available {
        out.push((0x25, PropType::Bool(self.retain_available)));
        // }
        if let Some(v) = self.maximum_packet_size {
            out.push((0x27, PropType::U32(v)));
        }
        // if let v = self.wildcard_subscription_available {
        out.push((0x28, PropType::Bool(self.wildcard_subscription_available)));
        // }
        // if let v = self.subscription_identifiers_available {
        out.push((
            0x29,
            PropType::Bool(self.subscription_identifiers_available),
        ));
        // }
        // if let v = self.shared_subscription_available {
        out.push((0x2A, PropType::Bool(self.shared_subscription_available)));
        // }
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// A struct for PUBACK, PUBCOMP, PUBREL and PUBREC
pub struct ConfirmationPacket {
    pub cmd: PacketType,
    /// Reason code is always 0 by default for MQTT 5
    /// but absent for MQTT 3 and 4
    pub puback_reason_code: Option<PubackPubrecCode>,
    pub pubcomp_reason_code: Option<PubcompPubrelCode>,
    pub properties: Option<ConfirmationProperties>,
    pub message_id: u16,
}

impl Properties for ConfirmationProperties {
    fn from_properties(props: Vec<(u8, PropType)>) -> Res<ConfirmationProperties> {
        let mut reason_string = None;
        let mut user_properties = UserProperties::new();
        for p in props {
            match p {
                (0x1F, PropType::String(v)) => reason_string = Some(v),
                (0x26, PropType::Map(v)) => user_properties = v,
                s => return Err(format!("Failed to parse confirmation properties {:?}", s)),
            }
        }
        Ok(ConfirmationProperties {
            reason_string,
            user_properties,
        })
    }

    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![];
        if let Some(s) = self.reason_string.as_ref() {
            out.push((0x1F, PropType::Str(s)));
        }
        if !self.user_properties.is_empty() {
            out.push((0x26, PropType::MapRef(&self.user_properties)));
        }
        Ok(out)
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
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

impl Properties for ConnectProperties {
    fn from_properties(props: Vec<(u8, PropType)>) -> Res<ConnectProperties> {
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

    fn to_pairs(&self) -> Res<Vec<(u8, PropType)>> {
        let mut out = vec![
            (0x11, PropType::U32(self.session_expiry_interval)),
            (0x21, PropType::U16(self.receive_maximum)),
        ];
        if let Some(v) = self.maximum_packet_size {
            out.push((0x27, PropType::U32(v)));
        }
        // if let v = self.topic_alias_maximum {
        out.push((0x22, PropType::U16(self.topic_alias_maximum)));
        // }
        // if let v = self.request_response_information {
        out.push((0x19, PropType::Bool(self.request_response_information)));
        // }
        // if let v = self.request_problem_information {
        out.push((0x17, PropType::Bool(self.request_problem_information)));
        // }
        // if let v = self.user_properties {
        out.push((0x26, PropType::MapRef(&self.user_properties)));
        // }
        if let Some(v) = self.authentication_method.as_ref() {
            out.push((0x15, PropType::Str(v)));
        }
        if let Some(v) = self.authentication_data.as_ref() {
            out.push((0x16, PropType::Str(v)));
        }
        Ok(out)
    }
}
