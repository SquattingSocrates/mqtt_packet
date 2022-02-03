use crate::structure::*;
use std::io::{BufRead, BufReader, Read};

static VARBYTEINT_MASK: u32 = 0x7F;
static VARBYTEINT_FIN_MASK: u32 = 0x80;

pub struct ByteReader<R: Read> {
    reader: BufReader<R>,
    curr_limit: Option<u32>,
    read_limits: Vec<u32>,
}

impl<R: Read> ByteReader<R> {
    pub fn new(reader: BufReader<R>) -> ByteReader<R> {
        ByteReader {
            reader,
            curr_limit: None,
            read_limits: vec![],
        }
    }

    pub fn read_header(&mut self) -> Result<(u32, FixedHeader), String> {
        // There is at least one byte in the buffer
        let first = self.read_u8()?;
        let fixed = FixedHeader::from_byte(first);
        // always read variable length to know how much we need to discard
        let length = self.read_variable_int();
        let fixed = match fixed {
            Err(e) => {
                if let Ok(len) = length {
                    self.take(len);
                    self.consume()?;
                    self.reset_limit();
                }
                return Err(e);
            }
            Ok(f) => f,
        };
        // unfortunately here we can't do anything but close the connection
        match length {
            Err(e) => Err(e),
            Ok(len) => {
                self.take(len);
                Ok((len, fixed))
            }
        }
    }

    /// sets a limit of reading, has to be used together
    /// with has_more() and reset_limit()
    pub fn take(&mut self, len: u32) {
        if let Some(l) = self.curr_limit {
            if len >= l {
                return;
            }
            // push alread subtracted number
            self.read_limits.push(l - len);
        }
        self.curr_limit = Some(len);
    }

    // this function tracks the curr_limit and imitates behaviour
    // of "Reader::take" while actually reading
    fn limit(&mut self, l: u32) {
        if let Some(len) = self.curr_limit {
            self.curr_limit = if len >= l { Some(len - l) } else { None };
        }
    }

    fn ensure_limit(&mut self, take_attempt: u32) -> Result<(), String> {
        if let Some(len) = self.curr_limit {
            if len < take_attempt {
                return Err(format!("Cannot take more than {}", len));
            }
        }
        Ok(())
    }

    /// resets limit if any was set
    pub fn reset_limit(&mut self) {
        let vals = (self.read_limits.pop(), self.curr_limit);
        match vals {
            // if no previous limit was None we just do a complete reset
            (None, _) => self.curr_limit = None,
            (x, None) => self.curr_limit = x,
            // check if current limit has not been reached
            // and add back unfinished reads
            (Some(l), Some(curr)) => self.curr_limit = Some(l + curr),
        }
    }

    pub fn read_len(&mut self, len: u32) -> Result<Vec<u8>, String> {
        self.ensure_limit(len)?;
        let mut buf = vec![0; len as usize];
        match self.reader.read_exact(&mut buf) {
            Ok(_) => {
                self.limit(len);
                Ok(buf)
            }
            Err(e) => Err(format!("Failed to read {} bytes. Reason: {:?}", len, e)),
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, String> {
        let d = self.read_len(1)?;
        if !d.is_empty() {
            Ok(d[0])
        } else {
            Err("Failed to read byte".to_string())
        }
    }

    pub fn read_u16(&mut self) -> Result<u16, String> {
        let d = self.read_len(2)?;
        if d.len() == 2 {
            Ok(((d[0] as u16) << 8) + d[1] as u16)
        } else {
            Err("Failed to read byte".to_string())
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, String> {
        let d = self.read_len(4)?;
        if d.len() == 4 {
            Ok(((d[0] as u32) << 24) + ((d[1] as u32) << 16) + ((d[2] as u32) << 8) + d[3] as u32)
        } else {
            Err("Failed to read byte".to_string())
        }
    }

    // reads utf-8 encoded strings with 2 bytes indicating length of string
    pub fn read_utf8_string(&mut self) -> Res<String> {
        let len = self.read_u16()?;
        match String::from_utf8(self.read_len(len as u32)?) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("Failed to read string: {:?}", e)),
        }
    }

    // reads binary data with prepending 2 bytes indicating length of string
    pub fn read_binary(&mut self) -> Res<Vec<u8>> {
        let len = self.read_u16()?;
        match self.read_len(len as u32) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("Failed to binary data: {:?}", e)),
        }
    }

    pub fn read_bool_byte(&mut self) -> Res<bool> {
        Ok(self.read_u8()? != 0)
    }

    /// read multibyte int and represent as u32 since they
    /// should not be longer than 4 bytes
    pub fn read_variable_int(&mut self) -> Result<u32, String> {
        let mut num = 0u32;
        let mut mult = 1;
        for _ in 0..4 {
            let next = self.read_len(1)?[0] as u32;
            num += mult * (next & VARBYTEINT_MASK);
            mult *= 0x80;
            if next & VARBYTEINT_FIN_MASK == 0 {
                break;
            }
        }
        if num > VARBYTEINT_MAX {
            return Err(format!("Invalid variable int {}", num));
        }
        Ok(num)
    }

    pub fn has_more(&mut self) -> bool {
        // if curr_limit is anything other than Some(0)
        // rely on fill_buf
        if let Some(0) = self.curr_limit {
            return false;
        }
        // TODO: switch to a better way to do this
        match self.reader.fill_buf() {
            Ok(v) => !v.is_empty(),
            Err(_) => false,
        }
    }

    fn start_properties_decode(&mut self) -> Res<u32> {
        let length = self.read_variable_int()?;
        if length > 0 {
            self.take(length);
        }
        Ok(length)
    }

    fn decode_property<'a, 'b>(&'a mut self) -> Result<(u8, PropType<'b>), String> {
        let prop_type = self.read_u8()?;
        match prop_type {
            0x02 | 0x18 | 0x11 | 0x27 => Ok((prop_type, PropType::U32(self.read_u32()?))),
            0x01 | 0x17 | 0x19 | 0x25 | 0x28 | 0x29 | 0x2A => {
                Ok((prop_type, PropType::Bool(self.read_bool_byte()?)))
            }
            0x03 | 0x08 | 0x15 | 0x16 | 0x12 | 0x1A | 0x1C | 0x1F => {
                Ok((prop_type, PropType::String(self.read_utf8_string()?)))
            }
            0x23 | 0x21 | 0x22 | 0x13 => Ok((prop_type, PropType::U16(self.read_u16()?))),
            0x09 => Ok((prop_type, PropType::Binary(self.read_binary()?))), // correlation data
            0x0B => Ok((prop_type, PropType::VarInt(self.read_variable_int()?))), // subscription identifier
            0x26 => {
                // user properties
                let name = self.read_utf8_string()?;
                let value = self.read_utf8_string()?;
                Ok((prop_type, PropType::Pair(name, value)))
            }
            0x24 => Ok((prop_type, PropType::U8(self.read_u8()?))),
            _ => Err(format!("Invalid property code: {}", prop_type)),
        }
    }

    pub fn consume(&mut self) -> Res<Vec<u8>> {
        if let Some(n) = self.curr_limit {
            self.read_len(n)
        } else {
            Err("Cannot consume if no limit specified".to_string())
        }
    }

    pub fn read_properties(&mut self) -> Res<Option<Vec<(u8, PropType)>>> {
        let mut props = vec![];
        // zero length properties are also valid
        if self.start_properties_decode()? == 0 {
            return Ok(None);
        }
        let mut user_properties = UserProperties::new();
        // let mut subscription_identifiers = vec![];

        // TODO: return Err when key is repeated, but not allowed to
        while self.has_more() {
            let prop = self.decode_property()?;
            match prop {
                (0x26, PropType::Pair(k, v)) => {
                    let p = user_properties.entry(k).or_insert_with(Vec::new);
                    p.push(v);
                }
                // parse variable byte int
                // (0x0B, PropType::U32(next)) => subscription_identifiers.push(next),
                x => props.push(x),
            }
        }

        if !user_properties.is_empty() {
            props.push((0x26, PropType::Map(user_properties)));
        }
        let _ = self.reset_limit();
        if props.is_empty() {
            return Ok(None);
        }
        Ok(Some(props))
    }
}
