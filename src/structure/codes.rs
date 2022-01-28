use super::common::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ReasonCode {}

pub trait MqttCode<T> {
    fn from_byte(byte: u8) -> Res<T>;
    fn to_byte(&self) -> u8;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// Use an enum to make setting the reason code easier and safer
pub enum SubscriptionReasonCode {
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
    SharedSubscriptionsNotSupported,
    /// 0xA1 The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifiersNotSupported,
    /// 0xA2 The Server does not support Wildcard Subscriptions; the subscription is not accepted.
    WildcardSubscriptionsNotSupported,
}

impl MqttCode<SubscriptionReasonCode> for SubscriptionReasonCode {
    fn from_byte(byte: u8) -> Res<SubscriptionReasonCode> {
        match byte {
            0x00 => Ok(SubscriptionReasonCode::GrantedQoS0),
            0x01 => Ok(SubscriptionReasonCode::GrantedQoS1),
            0x02 => Ok(SubscriptionReasonCode::GrantedQoS2),
            0x80 => Ok(SubscriptionReasonCode::UnspecifiedError),
            0x83 => Ok(SubscriptionReasonCode::ImplementationSpecificError),
            0x87 => Ok(SubscriptionReasonCode::NotAuthorized),
            0x8F => Ok(SubscriptionReasonCode::TopicFilterInvalid),
            0x91 => Ok(SubscriptionReasonCode::PacketIdentifierInUse),
            0x97 => Ok(SubscriptionReasonCode::QuotaExceeded),
            0x9E => Ok(SubscriptionReasonCode::SharedSubscriptionsNotSupported),
            0xA1 => Ok(SubscriptionReasonCode::SubscriptionIdentifiersNotSupported),
            0xA2 => Ok(SubscriptionReasonCode::WildcardSubscriptionsNotSupported),
            // fallback to unspecified error to keep function signature simple
            _ => Err(format!("Invalid suback code {}", byte)),
        }
    }

    fn to_byte(&self) -> u8 {
        match self {
            SubscriptionReasonCode::GrantedQoS0 => 0x00,
            SubscriptionReasonCode::GrantedQoS1 => 0x01,
            SubscriptionReasonCode::GrantedQoS2 => 0x02,
            SubscriptionReasonCode::UnspecifiedError => 0x80,
            SubscriptionReasonCode::ImplementationSpecificError => 0x83,
            SubscriptionReasonCode::NotAuthorized => 0x87,
            SubscriptionReasonCode::TopicFilterInvalid => 0x8F,
            SubscriptionReasonCode::PacketIdentifierInUse => 0x91,
            SubscriptionReasonCode::QuotaExceeded => 0x97,
            SubscriptionReasonCode::SharedSubscriptionsNotSupported => 0x9E,
            SubscriptionReasonCode::SubscriptionIdentifiersNotSupported => 0xA1,
            SubscriptionReasonCode::WildcardSubscriptionsNotSupported => 0xA2,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum DisconnectCode {
    NormalDisconnection,                 //0x00
    DisconnectWithWillMessage,           //0x04
    UnspecifiedError,                    //0x80
    MalformedPacket,                     //0x81
    ProtocolError,                       //0x82
    ImplementationSpecificError,         //0x83
    NotAuthorized,                       //0x87
    ServerBusy,                          //0x89
    ServerShuttingDown,                  //0x8B
    KeepAliveTimeout,                    //0x8D
    SessionTakenVver,                    //0x8E
    TopicFilterInvalid,                  //0x8F
    TopicNameInvalid,                    //0x90
    ReceiveMaximumExceeded,              //0x93
    TopicAliasInvalid,                   //0x94
    PacketTooLarge,                      //0x95
    MessageRateTooHigh,                  //0x96
    QuotaExceeded,                       //0x97
    AdministrativeAction,                //0x98
    PayloadFormatInvalid,                //0x99
    RetainNotSupported,                  //0x9A
    QoSNotSupported,                     //0x9B
    UseAnotherServer,                    //0x9C
    ServerMoved,                         //0x9D
    SharedSubscriptionsNotSupported,     //0x9E
    ConnectionRateExceeded,              //0x9F
    MaximumConnectTime,                  //0xA0
    SubscriptionIdentifiersNotSupported, //0xA1
    WildcardSubscriptionsNotSupported,   //0xA2
}

impl MqttCode<DisconnectCode> for DisconnectCode {
    fn from_byte(code: u8) -> Res<DisconnectCode> {
        Ok(match code {
            0x00 => DisconnectCode::NormalDisconnection, // 'Normal disconnection',
            0x04 => DisconnectCode::DisconnectWithWillMessage, // 'Disconnect with Will Message',
            0x80 => DisconnectCode::UnspecifiedError,    // 'Unspecified error',
            0x81 => DisconnectCode::MalformedPacket,     // 'Malformed Packet',
            0x82 => DisconnectCode::ProtocolError,       // 'Protocol Error',
            0x83 => DisconnectCode::ImplementationSpecificError, // 'Implementation specific error',
            0x87 => DisconnectCode::NotAuthorized,       // 'Not authorized',
            0x89 => DisconnectCode::ServerBusy,          // 'Server busy',
            0x8B => DisconnectCode::ServerShuttingDown,  // 'Server shutting down',
            0x8D => DisconnectCode::KeepAliveTimeout,    // 'Keep Alive timeout',
            0x8E => DisconnectCode::SessionTakenVver,    // 'Session taken over',
            0x8F => DisconnectCode::TopicFilterInvalid,  // 'Topic Filter invalid',
            0x90 => DisconnectCode::TopicNameInvalid,    // 'Topic Name invalid',
            0x93 => DisconnectCode::ReceiveMaximumExceeded, // 'Receive Maximum exceeded',
            0x94 => DisconnectCode::TopicAliasInvalid,   // 'Topic Alias invalid',
            0x95 => DisconnectCode::PacketTooLarge,      // 'Packet too large',
            0x96 => DisconnectCode::MessageRateTooHigh,  // 'Message rate too high',
            0x97 => DisconnectCode::QuotaExceeded,       // 'Quota exceeded',
            0x98 => DisconnectCode::AdministrativeAction, // 'Administrative action',
            0x99 => DisconnectCode::PayloadFormatInvalid, // 'Payload format invalid',
            0x9A => DisconnectCode::RetainNotSupported,  // 'Retain not supported',
            0x9B => DisconnectCode::QoSNotSupported,     // 'QoS not supported',
            0x9C => DisconnectCode::UseAnotherServer,    // 'Use another server',
            0x9D => DisconnectCode::ServerMoved,         // 'Server moved',
            0x9E => DisconnectCode::SharedSubscriptionsNotSupported, // 'Shared Subscriptions not supported',
            0x9F => DisconnectCode::ConnectionRateExceeded,          // 'Connection rate exceeded',
            0xA0 => DisconnectCode::MaximumConnectTime,              // 'Maximum connect time',
            0xA1 => DisconnectCode::SubscriptionIdentifiersNotSupported, // 'Subscription Identifiers not supported',
            0xA2 => DisconnectCode::WildcardSubscriptionsNotSupported, // 'Wildcard Subscriptions not supported'
            _ => return Err(format!("Invalid disconnect code {}", code)),
        })
    }

    fn to_byte(&self) -> u8 {
        match self {
            DisconnectCode::NormalDisconnection => 0x00, // 'Normal disconnection',
            DisconnectCode::DisconnectWithWillMessage => 0x04, // 'Disconnect with Will Message',
            DisconnectCode::UnspecifiedError => 0x80,    // 'Unspecified error',
            DisconnectCode::MalformedPacket => 0x81,     // 'Malformed Packet',
            DisconnectCode::ProtocolError => 0x82,       // 'Protocol Error',
            DisconnectCode::ImplementationSpecificError => 0x83, // 'Implementation specific error',
            DisconnectCode::NotAuthorized => 0x87,       // 'Not authorized',
            DisconnectCode::ServerBusy => 0x89,          // 'Server busy',
            DisconnectCode::ServerShuttingDown => 0x8B,  // 'Server shutting down',
            DisconnectCode::KeepAliveTimeout => 0x8D,    // 'Keep Alive timeout',
            DisconnectCode::SessionTakenVver => 0x8E,    // 'Session taken over',
            DisconnectCode::TopicFilterInvalid => 0x8F,  // 'Topic Filter invalid',
            DisconnectCode::TopicNameInvalid => 0x90,    // 'Topic Name invalid',
            DisconnectCode::ReceiveMaximumExceeded => 0x93, // 'Receive Maximum exceeded',
            DisconnectCode::TopicAliasInvalid => 0x94,   // 'Topic Alias invalid',
            DisconnectCode::PacketTooLarge => 0x95,      // 'Packet too large',
            DisconnectCode::MessageRateTooHigh => 0x96,  // 'Message rate too high',
            DisconnectCode::QuotaExceeded => 0x97,       // 'Quota exceeded',
            DisconnectCode::AdministrativeAction => 0x98, // 'Administrative action',
            DisconnectCode::PayloadFormatInvalid => 0x99, // 'Payload format invalid',
            DisconnectCode::RetainNotSupported => 0x9A,  // 'Retain not supported',
            DisconnectCode::QoSNotSupported => 0x9B,     // 'QoS not supported',
            DisconnectCode::UseAnotherServer => 0x9C,    // 'Use another server',
            DisconnectCode::ServerMoved => 0x9D,         // 'Server moved',
            DisconnectCode::SharedSubscriptionsNotSupported => 0x9E, // 'Shared Subscriptions not supported',
            DisconnectCode::ConnectionRateExceeded => 0x9F,          // 'Connection rate exceeded',
            DisconnectCode::MaximumConnectTime => 0xA0,              // 'Maximum connect time',
            DisconnectCode::SubscriptionIdentifiersNotSupported => 0xA1, // 'Subscription Identifiers not supported',
            DisconnectCode::WildcardSubscriptionsNotSupported => 0xA2, // 'Wildcard Subscriptions not supported'
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum UnsubackCode {
    Success,
    NoSubscriptionExisted,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicFilterInvalid,
    PacketIdentifierInUse,
}

impl MqttCode<UnsubackCode> for UnsubackCode {
    fn from_byte(code: u8) -> Res<UnsubackCode> {
        let c = match code {
            0x00 => UnsubackCode::Success,                     // 'Success',
            0x11 => UnsubackCode::NoSubscriptionExisted,       // 'No subscription existed',
            0x80 => UnsubackCode::UnspecifiedError,            // 'Unspecified error',
            0x83 => UnsubackCode::ImplementationSpecificError, // 'Implementation specific error',
            0x87 => UnsubackCode::NotAuthorized,               // 'Not authorized',
            0x8F => UnsubackCode::TopicFilterInvalid,          // 'Topic Filter invalid',
            0x91 => UnsubackCode::PacketIdentifierInUse,       // 'Packet Identifier in use'
            _ => return Err(format!("Invalid unsuback code {}", code)),
        };
        Ok(c)
    }

    fn to_byte(&self) -> u8 {
        match self {
            UnsubackCode::Success => 0x00,
            UnsubackCode::NoSubscriptionExisted => 0x11,
            UnsubackCode::UnspecifiedError => 0x80,
            UnsubackCode::ImplementationSpecificError => 0x83,
            UnsubackCode::NotAuthorized => 0x87,
            UnsubackCode::TopicFilterInvalid => 0x8F,
            UnsubackCode::PacketIdentifierInUse => 0x91,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AuthCode {
    Success,                // 0x0
    ContinueAuthentication, // 0x18
    ReAuthenticate,         // 0x19
}

impl MqttCode<AuthCode> for AuthCode {
    fn from_byte(byte: u8) -> Res<AuthCode> {
        Ok(match byte {
            0x00 => AuthCode::Success,
            0x18 => AuthCode::ContinueAuthentication,
            0x19 => AuthCode::ReAuthenticate,
            _ => return Err(format!("Invalid auth code {}", byte)),
        })
    }

    fn to_byte(&self) -> u8 {
        match self {
            AuthCode::Success => 0x00,
            AuthCode::ContinueAuthentication => 0x18,
            AuthCode::ReAuthenticate => 0x19,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// PUMBCOMP/PUBREL codes enum
pub enum PubcompPubrelCode {
    Success,                  // 0x0
    PacketIdentifierNotFound, // 0x92
}

impl MqttCode<PubcompPubrelCode> for PubcompPubrelCode {
    fn from_byte(byte: u8) -> Res<PubcompPubrelCode> {
        Ok(match byte {
            0x00 => PubcompPubrelCode::Success,
            0x92 => PubcompPubrelCode::PacketIdentifierNotFound,
            _ => return Err(format!("Invalid pubcomp/pubrel code {}", byte)),
        })
    }

    fn to_byte(&self) -> u8 {
        match self {
            PubcompPubrelCode::Success => 0x00,
            PubcompPubrelCode::PacketIdentifierNotFound => 0x92,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
/// PUBACK/PUBREC codes enum
pub enum PubackPubrecCode {
    Success,
    NoMatchingSubscribers,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicNameInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    PayloadFormatInvalid,
}

impl MqttCode<PubackPubrecCode> for PubackPubrecCode {
    fn from_byte(byte: u8) -> Res<PubackPubrecCode> {
        Ok(match byte {
            0x00 => PubackPubrecCode::Success,
            0x10 => PubackPubrecCode::NoMatchingSubscribers,
            0x80 => PubackPubrecCode::UnspecifiedError,
            0x83 => PubackPubrecCode::ImplementationSpecificError,
            0x87 => PubackPubrecCode::NotAuthorized,
            0x90 => PubackPubrecCode::TopicNameInvalid,
            0x91 => PubackPubrecCode::PacketIdentifierInUse,
            0x97 => PubackPubrecCode::QuotaExceeded,
            0x99 => PubackPubrecCode::PayloadFormatInvalid,
            _ => return Err(format!("Invalid puback/pubrec code {}", byte)),
        })
    }

    fn to_byte(&self) -> u8 {
        match self {
            PubackPubrecCode::Success => 0x00,
            PubackPubrecCode::NoMatchingSubscribers => 0x10,
            PubackPubrecCode::UnspecifiedError => 0x80,
            PubackPubrecCode::ImplementationSpecificError => 0x83,
            PubackPubrecCode::NotAuthorized => 0x87,
            PubackPubrecCode::TopicNameInvalid => 0x90,
            PubackPubrecCode::PacketIdentifierInUse => 0x91,
            PubackPubrecCode::QuotaExceeded => 0x97,
            PubackPubrecCode::PayloadFormatInvalid => 0x99,
        }
    }
}
