use crate::byte_reader::*;
use crate::byte_reader::*;
use crate::connect::*;
use crate::structure::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[macro_use]
macro_rules! extend_base_packet {
    (pub struct $name:ident { $( $field:ident: $ty:ty ),* $(,)* }) => {
        struct $name {
            fixed: FixedHeader,
            message_id: Option<u32>,
            $( $field: $ty ),*
        }
    };
}

pub enum MqttPacket {
    Connect(ConnectPacket),
}

fn is_variable_length_int(byte: &u8) -> bool {
    *byte & 0x40 == 0x40
}

// fn decode_bytes(buf: &[u8]) -> Self;
// fn encode_bytes(&self) -> Vec<u8>;

struct PacketParser {}

impl PacketParser {
    // helper method

    // pub fn read_next(&mut self) -> MqttPacket {
    //     let (length, fixed) = self.reader.decode_header()?;
    //     match fixed.cmd {
    //         Connect => MqttPacket::Connect(ConnectPacket::decode_bytes(self.reader, fixed, length)),
    //     }
    // }
}

// export interface IPublishPacket extends IPacket {
//   cmd: 'publish'
//   qos: QoS
//   dup: boolean
//   retain: boolean
//   topic: string
//   payload: string | Buffer
//   properties?: {
//     payloadFormatIndicator: bool,
//     messageExpiryInterval?: number,
//     topicAlias?: number,
//     responseTopic: Option<String>,
//     correlationData?: Buffer,
//     userProperties?: UserProperties,
//     subscriptionIdentifier?: number,
//     contentType: Option<String>,
//   }
// }

// export interface IConnackPacket extends IPacket {
//   cmd: 'connack'
//   returnCode?: number,
//   reasonCode?: number,
//   sessionPresent: boolean
//   properties?: {
//     sessionExpiryInterval?: number,
//     receiveMaximum?: number,
//     maximumQoS?: number,
//     retainAvailable: bool,
//     maximumPacketSize?: number,
//     assignedClientIdentifier: Option<String>,
//     topicAliasMaximum?: number,
//     reasonString: Option<String>,
//     userProperties?: UserProperties,
//     wildcardSubscriptionAvailable: bool,
//     subscriptionIdentifiersAvailable: bool,
//     sharedSubscriptionAvailable: bool,
//     serverKeepAlive?: number,
//     responseInformation: Option<String>,
//     serverReference: Option<String>,
//     authenticationMethod: Option<String>,
//     authenticationData?: Buffer
//   }
// }

// export interface ISubscription {
//   topic: string
//   qos: QoS,
//   nl: bool,
//   rap: bool,
//   rh?: number
// }

// export interface ISubscribePacket extends IPacket {
//   cmd: 'subscribe'
//   subscriptions: ISubscription[],
//   properties?: {
//     reasonString: Option<String>,
//     userProperties?: UserProperties
//   }
// }

// export interface ISubackPacket extends IPacket {
//   cmd: 'suback',
//   reasonCode?: number,
//   properties?: {
//     reasonString: Option<String>,
//     userProperties?: UserProperties
//   },
//   granted: number[] | Object[]
// }

// export interface IUnsubscribePacket extends IPacket {
//   cmd: 'unsubscribe',
//   properties?: {
//     reasonString: Option<String>,
//     userProperties?: UserProperties
//   },
//   unsubscriptions: string[]
// }

// export interface IUnsubackPacket extends IPacket {
//   cmd: 'unsuback',
//   reasonCode?: number,
//   properties?: {
//     reasonString: Option<String>,
//     userProperties?: UserProperties
//   }
// }

// export interface IPubackPacket extends IPacket {
//   cmd: 'puback',
//   reasonCode?: number,
//   properties?: {
//     reasonString: Option<String>,
//     userProperties?: UserProperties
//   }
// }

// export interface IPubcompPacket extends IPacket {
//   cmd: 'pubcomp',
//   reasonCode?: number,
//   properties?: {
//     reasonString: Option<String>,
//     userProperties?: UserProperties
//   }
// }

// export interface IPubrelPacket extends IPacket {
//   cmd: 'pubrel',
//   reasonCode?: number,
//   properties?: {
//     reasonString: Option<String>,
//     userProperties?: UserProperties
//   }
// }

// export interface IPubrecPacket extends IPacket {
//   cmd: 'pubrec',
//   reasonCode?: number,
//   properties?: {
//     reasonString: Option<String>,
//     userProperties?: UserProperties
//   }
// }

// export interface IPingreqPacket extends IPacket {
//   cmd: 'pingreq'
// }

// export interface IPingrespPacket extends IPacket {
//   cmd: 'pingresp'
// }

// export interface IDisconnectPacket extends IPacket {
//   cmd: 'disconnect',
//   reasonCode?: number,
//   properties?: {
//     sessionExpiryInterval?: number,
//     reasonString: Option<String>,
//     userProperties?: UserProperties,
//     serverReference: Option<String>,
//   }
// }
