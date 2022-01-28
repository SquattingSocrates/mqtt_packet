use crate::packet::*;
use crate::structure::*;

const MQISDP_BUF: [u8; 6] = [b'M', b'Q', b'I', b's', b'd', b'p'];
const MQTT_BUF: [u8; 4] = [b'M', b'Q', b'T', b'T'];

impl PacketEncoder {
  pub fn encode_connect(mut self, packet: ConnectPacket) -> Res<Vec<u8>> {
    let ConnectPacket {
      properties,
      protocol_id,
      protocol_version,
      password,
      fixed,
      client_id,
      will,
      clean_session,
      keep_alive,
      user_name,
      ..
    } = packet;
    let mut length = 0;

    // add protocol length
    length += 2
      + match protocol_id {
        Protocol::Mqtt => 4,
        Protocol::MQIsdp => 6,
      };

    // Must be 3 or 4 or 5
    if let 3 | 4 | 5 = protocol_version {
      length += 1;
    } else {
      return Err("Invalid protocol version".to_string());
    }

    // ClientId might be omitted in 3.1.1 and 5, but only if cleanSession is set to 1
    if (client_id.is_empty() && protocol_version >= 4 && clean_session) || !client_id.is_empty() {
      length += client_id.len() + 2;
    } else {
      if protocol_version < 4 {
        return Err("client_id must be supplied before 3.1.1".to_string());
      }
      if !clean_session {
        return Err("client_id must be given if clean_session set to false".to_string());
      }
    }

    // "keep_alive" Must be a two byte number
    // also add connect flags
    length += 2 + 1;

    // mqtt5 properties
    let properties_data = PropertyEncoder::encode(properties, protocol_version)?;
    length += properties_data.len();

    // If will exists...
    let mut will_retain = false;
    let mut will_qos = None;
    let mut has_will = false;
    let mut will_properties = vec![];
    let mut will_topic = String::new();
    let mut will_payload = String::new();
    if let Some(will) = will {
      let LastWill {
        topic,
        payload,
        properties,
        qos,
        retain,
      } = will;
      has_will = true;
      will_retain = retain;
      will_qos = Some(qos);
      // It must have non-empty topic
      // add topic length if any
      if let Some(t) = topic {
        if t.is_empty() {
          return Err("Not allowed to use empty will topic".to_string());
        }
        will_topic = t.clone();
        length += t.len() + 2;
      }

      // Payload
      length += 2; // payload length
      if let Some(data) = payload {
        will_payload = data.clone();
        length += data.len();
      }
      // will properties
      if protocol_version == 5 {
        will_properties = PropertyEncoder::encode(properties, protocol_version)?;
        length += will_properties.len();
        // } else {
        //     vec![0]
        // }
      }
    }

    // Username
    let mut has_username = false;
    if let Some(user_name) = &user_name {
      has_username = true;
      length += user_name.len() + 2;
    }

    // Password
    let mut has_password = false;
    if let Some(pass) = &password {
      if !has_username {
        return Err("Username is required to use password".to_string());
      }
      has_password = true;
      length += pass.len() + 2;
    }

    // write header
    self.buf.push(fixed.encode());
    // length
    self.write_variable_num(length as u32)?;
    // protocol id and protocol version
    let proto_vec = match protocol_id {
      Protocol::MQIsdp => MQISDP_BUF.to_vec(),
      Protocol::Mqtt => MQTT_BUF.to_vec(),
    };
    self.write_u16(proto_vec.len() as u16);
    self.write_vec(proto_vec);
    self.write_u8(protocol_version);
    // write connect flags
    self.write_u8(
      ((has_username as u8) * 0x80) //user_name:  0x80 = (1 << 7)
      | ((has_password as u8) * 0x40) //password:  0x40 = (1 << 6)
      | ((will_retain as u8) * 0x20)  //will_retain:  0x20 = (1 << 5)
      | ((will_qos.unwrap_or(0) << 3) & 0x18)     //will_qos:  0x18 = 24 = ((1 << 4) + (1 << 3)),
      | ((has_will as u8) * 0x4) //will:  0x4 = 1 << 2
      | ((clean_session as u8) * 0x2), //clean_session:  0x2 = 1 << 2)
    );
    // write keep alive
    self.write_u16(keep_alive);

    self.write_vec(properties_data);
    // client id
    self.write_utf8_string(client_id);
    // will properties
    if protocol_version == 5 {
      self.write_vec(will_properties);
    }
    // will topic and payload
    if has_will {
      self.write_utf8_string(will_topic);
      self.write_utf8_string(will_payload);
    }

    // username
    if let Some(u) = user_name {
      self.write_utf8_string(u);
    }
    // password
    if let Some(p) = password {
      self.write_utf8_string(p);
    }
    Ok(self.buf)
  }
}
