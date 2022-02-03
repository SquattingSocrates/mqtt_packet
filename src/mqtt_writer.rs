use crate::structure::{FixedHeader, PropType, Res, VARBYTEINT_MAX};

pub struct MqttWriter {
    pub(crate) buf: Vec<u8>,
}

impl MqttWriter {
    /// since the length of the packet should be known
    /// we can initialize a write with the correct capacity right aways
    pub fn new(length: usize) -> MqttWriter {
        MqttWriter {
            buf: Vec::with_capacity(length),
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    pub fn write_properties(&mut self, props: Vec<(u8, PropType)>) -> Res<()> {
        for prop in props {
            match prop {
                (code, PropType::U32(v)) => {
                    self.write_u8(code);
                    self.write_u32(v);
                }
                (code, PropType::U16(v)) => {
                    self.write_u8(code);
                    self.write_u16(v)
                }
                (code, PropType::U8(v)) => {
                    self.write_u8(code);
                    self.write_u8(v)
                }
                (code, PropType::String(v)) => {
                    self.write_u8(code);
                    self.write_utf8_string(v)
                }
                (code, PropType::Str(v)) => {
                    self.write_u8(code);
                    self.write_utf8_str(v)
                }
                (code, PropType::Binary(v)) => {
                    self.write_u8(code);
                    self.write_binary(v)
                }
                (code, PropType::BinaryRef(v)) => {
                    self.write_u8(code);
                    self.write_binary_ref(v)
                }
                // should never happen actually
                (_, PropType::Pair(_, _)) => {}
                // write code code and two strings for each key-value
                // pair
                (code, PropType::Map(map)) => {
                    for (k, v) in map.into_iter() {
                        // split into pairs
                        for val in v {
                            self.write_u8(code);
                            self.write_utf8_str(&k);
                            self.write_utf8_str(&val);
                        }
                    }
                }
                (code, PropType::MapRef(map)) => {
                    for (k, v) in map.iter() {
                        // split into pairs
                        for val in v {
                            self.write_u8(code);
                            self.write_utf8_str(k);
                            self.write_utf8_str(val);
                        }
                    }
                }
                (code, PropType::VarInt(num)) => {
                    self.write_u8(code);
                    self.write_variable_num(num)?;
                }
                (code, PropType::Bool(v)) => {
                    self.write_u8(code);
                    self.write_u8(v as u8)
                }
                (code, PropType::U32Vec(v)) => {
                    self.write_u8(code);
                    for num in v {
                        self.write_u32(num);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn encode_multibyte_num(message_id: u32) -> Vec<u8> {
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

    pub fn write_variable_num(&mut self, num: u32) -> Res<()> {
        if num > VARBYTEINT_MAX {
            return Err(format!("Invalid variable int {}", num));
        }
        let mut encoded = if num == 0 {
            vec![0]
        } else {
            Self::encode_variable_num(num)
        };
        self.buf.append(&mut encoded);
        Ok(())
    }

    pub fn write_utf8_string(&mut self, s: String) {
        self.write_u16(s.len() as u16);
        for b in s.bytes() {
            self.buf.push(b);
        }
    }

    pub fn write_utf8_str(&mut self, s: &str) {
        self.write_u16(s.len() as u16);
        for b in s.bytes() {
            self.buf.push(b);
        }
    }

    /// a Binary vector should never be empty
    pub fn write_binary(&mut self, s: Vec<u8>) {
        self.write_u16(s.len() as u16);
        self.write_vec(s);
    }

    pub fn write_binary_ref(&mut self, s: &[u8]) {
        self.write_u16(s.len() as u16);
        self.write_slice(s);
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

    pub fn write_slice(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
    }

    pub fn write_sized(&mut self, v: &[u8], size: &[u8]) -> Res<()> {
        // write only 0 to indicate empty properties
        if v.is_empty() && size == [0] {
            self.write_u8(0);
        } else if !v.is_empty() {
            // normally encode length and properties
            self.write_slice(size);
            self.write_slice(v);
        }
        Ok(())
    }
}
