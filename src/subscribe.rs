use crate::byte_reader::ByteReader;
use crate::mqtt_writer::MqttWriter;
use crate::structure::*;
use std::io;

const SUBSCRIBE_OPTIONS_NL_MASK: u8 = 0x01;
const SUBSCRIBE_OPTIONS_NL_SHIFT: u8 = 2;
const SUBSCRIBE_OPTIONS_RAP_MASK: u8 = 0x01;
const SUBSCRIBE_OPTIONS_RAP_SHIFT: u8 = 3;
const SUBSCRIBE_OPTIONS_RH_MASK: u8 = 0x03;
const SUBSCRIBE_OPTIONS_RH_SHIFT: u8 = 4;

impl Packet for SubscribePacket {
    fn decode<R: io::Read>(
        reader: &mut ByteReader<R>,
        fixed: FixedHeader,
        length: u32,
        protocol_version: u8,
    ) -> Res<Self> {
        let message_id = reader.read_u16()?;

        let mut packet = SubscribePacket {
            qos: fixed.qos,
            properties: None,
            message_id,
            subscriptions: vec![],
        };

        // Properties mqtt 5
        if protocol_version == 5 {
            packet.properties = match reader.read_properties()? {
                None => None,
                Some(props) => Some(SubscribeProperties::from_properties(props)?),
            };
        }

        if !reader.has_more() {
            return Err("Malformed subscribe, no payload specified".to_string());
        }

        while reader.has_more() {
            // Parse topic
            let topic = reader.read_utf8_string()?;
            let options = reader.read_u8()?;

            if protocol_version == 5 {
                if options & 0xc0 > 0 {
                    return Err("Invalid subscribe topic flag bits, bits 7-6 must be 0".to_string());
                }
            } else if options & 0xfc > 0 {
                return Err("Invalid subscribe topic flag bits, bits 7-2 must be 0".to_string());
            }

            let qos = match options & 0x03 {
                0 => QoS::QoS0,
                1 => QoS::QoS1,
                2 => QoS::QoS2,
                _ => return Err("Invalid subscribe QoS, must be <= 2".to_string()),
            };

            let mut subscription = Subscription {
                topic,
                qos,
                nl: false,
                rap: false,
                rh: None,
            };

            // mqtt 5 options
            if protocol_version == 5 {
                subscription.nl =
                    ((options >> SUBSCRIBE_OPTIONS_NL_SHIFT) & SUBSCRIBE_OPTIONS_NL_MASK) != 0;
                subscription.rap =
                    ((options >> SUBSCRIBE_OPTIONS_RAP_SHIFT) & SUBSCRIBE_OPTIONS_RAP_MASK) != 0;
                subscription.rh =
                    match (options >> SUBSCRIBE_OPTIONS_RH_SHIFT) & SUBSCRIBE_OPTIONS_RH_MASK {
                        rh @ (0 | 1 | 2) => Some(rh),
                        _ => return Err("Invalid retain handling, must be <= 2".to_string()),
                    };
            }
            // TODO: include once bridge_mode is implemented
            /*else if bridge_mode {
              subscription.rh = 0
              subscription.rap = true
              subscription.nl = true
            }*/

            // Push pair to subscriptions
            packet.subscriptions.push(subscription)
        }
        Ok(packet)
    }

    fn encode(&self, protocol_version: u8) -> Res<Vec<u8>> {
        // Check message ID
        let mut length = 2;

        // check subscriptions
        for sub in self.subscriptions.iter() {
            if sub.topic.is_empty() {
                return Err("Invalid subscriptions - empty topic".to_string());
            }

            if protocol_version == 5 && (sub.rh.is_none() || sub.rh.unwrap() > 2) {
                return Err("Invalid subscriptions - invalid Retain Handling".to_string());
            }

            length += sub.topic.len() + 2 + 1;
        }

        // properies mqtt 5
        let properties_data =
            Properties::encode_option(self.properties.as_ref(), protocol_version)?;
        length += properties_data.len();
        let mut writer = MqttWriter::new(length);
        // header
        writer.write_header(FixedHeader {
            cmd: PacketType::Unsuback,
            qos: self.qos,
            dup: false,
            retain: false,
        });

        // Length
        writer.write_variable_num(length as u32)?;

        // Message ID
        writer.write_u16(self.message_id);

        // properies mqtt 5
        writer.write_vec(properties_data);

        // subscriptions payload
        for sub in self.subscriptions.iter() {
            writer.write_utf8_str(&sub.topic);
            let mut options = sub.qos.to_byte();
            if protocol_version == 5 {
                let nl = (sub.nl as u8) << SUBSCRIBE_OPTIONS_NL_SHIFT;
                let rap = (sub.rap as u8) << SUBSCRIBE_OPTIONS_RAP_SHIFT;
                let rh = match sub.rh {
                    Some(0 | 1 | 2) => sub.rh.unwrap() << SUBSCRIBE_OPTIONS_RH_SHIFT,
                    _ => return Err("Invalid retain handling, must be <= 2".to_string()),
                };
                options = options | nl | rap | rh;
            }
            writer.write_u8(options);
        }
        Ok(writer.into_vec())
    }
}
