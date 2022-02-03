mod codes;
mod common;
mod properties;
use crate::byte_reader::ByteReader;
pub use codes::*;
pub use common::*;
pub use properties::*;
#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};
use std::io;

pub trait Packet: Sized {
    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>>;
    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<Self>;
    // fn matches(t: PacketType) -> bool;
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct AuthPacket {
    pub reason_code: AuthCode,
    pub properties: Option<AuthProperties>,
}

#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct LastWill {
    pub topic: Option<String>,
    pub payload: Option<String>,
    pub qos: u8,
    pub retain: bool,
    pub properties: Option<WillProperties>,
}

#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ConnectPacket {
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

#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ConnackPacket {
    pub return_code: Option<u8>,
    pub reason_code: Option<u8>,
    pub session_present: bool,
    pub properties: Option<ConnackProperties>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct DisconnectPacket {
    // only exists in MQTT 5
    pub reason_code: Option<DisconnectCode>,
    pub properties: Option<DisconnectProperties>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Captures value of published message
pub struct PublishPacket {
    pub dup: bool,
    pub qos: u8,
    pub retain: bool,
    pub topic: String,
    pub message_id: Option<u16>,
    /// No assumptions are made about the structure
    /// and content of payload
    pub payload: Vec<u8>,
    /// Used in MQTT 5
    pub properties: Option<PublishProperties>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Subscription {
    /// Name of topic or wildcard pattern to subscribe to
    pub topic: String,
    /// Requested max QoS
    pub qos: QoS,
    /// NL = No local
    /// Bit 2 of the Subscription Options represents the No Local option.
    /// If the value is 1, Application Messages MUST NOT be forwarded to a
    /// connection with a ClientID equal to the ClientID of the publishing
    /// connection [MQTT-3.8.3-3]. It is a Protocol Error to set the No Local
    /// bit to 1 on a Shared Subscription
    pub nl: bool,
    /// Bit 3 of the Subscription Options represents the Retain As Published
    /// option. If 1, Application Messages forwarded using this subscription
    /// keep the RETAIN flag they were published with. If 0, Application
    /// Messages forwarded using this subscription have the RETAIN flag
    /// set to 0. Retained messages sent when the subscription is established
    /// have the RETAIN flag set to 1.
    pub rap: bool,
    /// Bits 4 and 5 of the Subscription Options represent the Retain Handling
    /// option. This option specifies whether retained messages are sent when
    /// the subscription is established. This does not affect the sending of
    /// retained messages at any point after the subscribe. If there are no
    /// retained messages matching the Topic Filter, all of these values act the same.
    /// The values are:
    ///
    /// 0 = Send retained messages at the time of the subscribe
    ///
    /// 1 = Send retained messages at subscribe only if the subscription does not currently exist
    ///
    /// 2 = Do not send retained messages at the time of the subscribe
    pub rh: Option<u8>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct SubscribePacket {
    pub qos: u8,
    pub subscriptions: Vec<Subscription>,
    pub properties: Option<SubscribeProperties>,
    pub message_id: u16,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Packet that holds information of subscription acknowledgement (SUBACK)
pub struct SubackPacket {
    pub reason_code: Option<u8>,
    pub message_id: u16,
    pub properties: Option<ConfirmationProperties>,
    /// used in MQTT 5
    pub granted_reason_codes: Vec<SubscriptionReasonCode>,
    /// used in MQTT 3.1 and 4
    pub granted: Vec<Granted>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct UnsubscribePacket {
    pub qos: u8,
    pub message_id: u16,
    pub properties: Option<UnsubscribeProperties>,
    pub unsubscriptions: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct UnsubackPacket {
    pub message_id: u16,
    /// used only in MQTT 5, will always empty if
    /// not MQTT 5
    pub granted: Vec<UnsubackCode>,
    pub properties: Option<ConfirmationProperties>,
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct PingreqPacket;

impl Packet for PingreqPacket {
    fn decode<R: io::Read>(_: &mut ByteReader<R>, _: FixedHeader, _: u32, _: u8) -> Res<Self> {
        Ok(PingreqPacket {})
    }

    fn encode(&self, _: u8) -> Res<Vec<u8>> {
        Ok(vec![
            FixedHeader::encode(&FixedHeader::for_type(PacketType::Pingreq)),
            0,
        ])
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct PingrespPacket;

impl Packet for PingrespPacket {
    fn decode<R: io::Read>(_: &mut ByteReader<R>, _: FixedHeader, _: u32, _: u8) -> Res<Self> {
        Ok(PingrespPacket {})
    }

    fn encode(&self, _: u8) -> Res<Vec<u8>> {
        Ok(vec![
            FixedHeader::encode(&FixedHeader::for_type(PacketType::Pingresp)),
            0,
        ])
    }
}
