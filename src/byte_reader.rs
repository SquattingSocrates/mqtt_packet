use crate::structure::*;
use std::io::{BufRead, BufReader, Read};

static VARBYTEINT_MASK: u32 = 0x7F;
static VARBYTEINT_FIN_MASK: u32 = 0x80;
static VARBYTEINT_MAX: u32 = 268435455;

pub struct ByteReader<R: Read> {
    reader: BufReader<R>,
    read_limit: Option<u32>,
}

impl<R: Read> ByteReader<R> {
    pub fn new(reader: BufReader<R>) -> ByteReader<R> {
        ByteReader {
            reader,
            read_limit: None,
        }
    }

    pub fn read_header(&mut self) -> Result<(u32, FixedHeader), String> {
        // There is at least one byte in the buffer
        let first = self.read_u8()?;
        let fixed = FixedHeader::from_byte(first)?;
        let length = self.read_variable_int()?;
        Ok((length, fixed))
    }

    /// sets a limit of reading, has to be used together
    /// with has_more() and reset_limit()
    pub fn take(&mut self, len: u32) {
        self.read_limit = Some(len);
    }

    // this function tracks the read_limit and imitates behaviour
    // of "Reader::take" while actually reading
    fn limit(&mut self, l: u32) {
        if let Some(len) = self.read_limit {
            self.read_limit = if len >= l { Some(len - l) } else { None };
        }
    }

    fn ensure_limit(&mut self, take_attempt: u32) -> Result<(), String> {
        if let Some(len) = self.read_limit {
            if len < take_attempt {
                return Err(format!("Cannot take more than {}", len));
            }
        }
        Ok(())
    }

    /// resets limit if any was set
    pub fn reset_limit(&mut self) {
        self.read_limit = None;
    }

    pub fn read_len(&mut self, len: u32) -> Result<Vec<u8>, String> {
        self.ensure_limit(len)?;
        let mut buf = vec![0; len as usize];
        match self.reader.read_exact(&mut buf) {
            Ok(_) => {
                // println!("READ_LEN({}) = {:?}", len, buf);
                self.limit(len);
                Ok(buf)
            }
            Err(e) => Err(format!("Failed to read {} bytes. Reason: {:?}", len, e)),
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, String> {
        let d = self.read_len(1)?;
        if d.len() > 0 {
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
    pub fn read_utf8_string(&mut self) -> Result<String, String> {
        let len = self.read_len(2)?;
        let len = ((len[0] as u16) << 8) + (len[1] as u16);
        match String::from_utf8(self.read_len(len as u32)?) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("Failed to read string: {:?}", e)),
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
        Ok(num)
    }

    pub fn has_more(&mut self) -> bool {
        // if read_limit is anything other than Some(0)
        // rely on fill_buf
        if let Some(0) = self.read_limit {
            return false;
        }
        // TODO: switch to a better way to do this
        match self.reader.fill_buf() {
            Ok(v) => !v.is_empty(),
            Err(_) => false,
        }
    }

    fn start_properties_decode(&mut self) -> Res<()> {
        let length = self.read_variable_int()?;
        self.take(length);
        Ok(())
    }

    fn decode_property(&mut self) -> Result<(u8, PropType), String> {
        let prop_type = self.read_u8()?;
        match prop_type {
            0x01 => Ok((prop_type, PropType::Bool(self.read_bool_byte()?))),
            0x02 => Ok((prop_type, PropType::U32(self.read_u32()?))),
            0x03 => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x08 => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x09 => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x0B => Ok((prop_type, PropType::U32(self.read_u32()?))),
            0x23 => Ok((prop_type, PropType::U16(self.read_u16()?))),
            0x26 => {
                let name = self.read_utf8_string()?;
                let value = self.read_utf8_string()?;
                Ok((prop_type, PropType::Pair(name, value)))
            }
            0x18 => Ok((prop_type, PropType::U32(self.read_u32()?))),
            0x11 => Ok((prop_type, PropType::U32(self.read_u32()?))),
            0x15 => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x16 => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x17 => Ok((prop_type, PropType::Bool(self.read_bool_byte()?))),
            0x19 => Ok((prop_type, PropType::Bool(self.read_bool_byte()?))),
            0x21 => Ok((prop_type, PropType::U16(self.read_u16()?))),
            0x22 => Ok((prop_type, PropType::U16(self.read_u16()?))),
            0x27 => Ok((prop_type, PropType::U32(self.read_u32()?))),
            0x12 => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x13 => Ok((prop_type, PropType::U16(self.read_u16()?))),
            0x1A => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x1C => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x1F => Ok((prop_type, PropType::String(self.read_utf8_string()?))),
            0x24 => Ok((prop_type, PropType::Bool(self.read_bool_byte()?))),
            0x25 => Ok((prop_type, PropType::Bool(self.read_bool_byte()?))),
            0x28 => Ok((prop_type, PropType::Bool(self.read_bool_byte()?))),
            0x29 => Ok((prop_type, PropType::Bool(self.read_bool_byte()?))),
            0x2A => Ok((prop_type, PropType::Bool(self.read_bool_byte()?))),
            _ => Err("Invalid property code".to_string()),
        }
    }

    pub fn read_properties(&mut self) -> Res<Vec<(u8, PropType)>> {
        let mut props = vec![];
        self.start_properties_decode()?;
        let mut user_properties = UserProperties::new();

        while self.has_more() {
            let prop = self.decode_property()?;
            match prop {
                (0x26, PropType::Pair(k, v)) => {
                    let p = user_properties.entry(k).or_insert(vec![]);
                    p.push(v);
                }
                x => props.push(x),
            }
        }

        if user_properties.len() > 0 {
            props.push((0x26, PropType::Map(user_properties)));
        }
        self.reset_limit();
        Ok(props)
    }
}

mod test {
    use crate::byte_reader::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_read_u8() {
        let src = Cursor::new(vec![1, 2, 3]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok(1), reader.read_u8());
        assert_eq!(Ok(2), reader.read_u8());
        assert_eq!(Ok(3), reader.read_u8());
    }

    #[test]
    fn test_read_u16() {
        let src = Cursor::new(vec![1, 1]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok(0x0101), reader.read_u16());
    }

    #[test]
    fn test_read_u32() {
        let src = Cursor::new(vec![0, 128, 1, 128]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok(0x800180), reader.read_u32());
    }

    #[test]
    fn test_read_string() {
        let src = Cursor::new(vec![0u8, 4u8, b'M', b'Q', b'T', b'T']);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok("MQTT".to_string()), reader.read_utf8_string());
    }

    #[test]
    fn test_variable_int() {
        // 1 byte
        let src = Cursor::new(vec![13]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok(13), reader.read_variable_int());
        // 2 bytes
        let src = Cursor::new(vec![203, 13]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok(1739), reader.read_variable_int());
        // 3 bytes
        let src = Cursor::new(vec![216, 203, 13]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok(222_680), reader.read_variable_int());
        // 4 bytes
        let src = Cursor::new(vec![198, 216, 203, 13]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok(28_503_110), reader.read_variable_int());
    }

    #[test]
    fn test_has_more() {
        let src = Cursor::new(vec![0, 4, 8]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        assert_eq!(Ok(0), reader.read_u8());
        assert!(reader.has_more());
        // should not really "read"/advance
        assert_eq!(Ok(4), reader.read_u8());
        assert_eq!(Ok(8), reader.read_u8());
        // should not have more
        assert!(!reader.has_more());
    }

    #[test]
    fn test_take() {
        let src = Cursor::new(vec![0, 4, 8, 16, 32]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        reader.take(2);
        assert_eq!(Ok(0), reader.read_u8());
        assert!(reader.has_more());
        assert_eq!(Ok(4), reader.read_u8());
        assert_eq!(false, reader.has_more());
        // should not really stop
        reader.reset_limit();
        assert_eq!(Ok(8), reader.read_u8());
        assert_eq!(Ok(16), reader.read_u8());
        assert_eq!(Ok(32), reader.read_u8());
        // should not have more
        assert!(!reader.has_more());
    }
}
