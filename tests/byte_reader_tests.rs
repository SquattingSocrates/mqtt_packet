mod test {
    use mqtt_packet_3_5::byte_reader::ByteReader;
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

    #[test]
    // this is important since we might be
    fn test_multiple_limits() {
        let src = Cursor::new(vec![0, 4, 8, 16, 32, 64, 128]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        reader.take(5);
        assert_eq!(Ok(0), reader.read_u8());
        // take less now, 4 were left
        reader.take(3);
        assert!(reader.has_more());
        assert_eq!(Ok(4), reader.read_u8());
        assert!(reader.has_more());
        assert_eq!(Ok(8), reader.read_u8());
        assert!(reader.has_more());
        // reset limit to 5 - 3 + 1
        reader.reset_limit();
        assert_eq!(Ok(16), reader.read_u8());
        assert_eq!(Ok(32), reader.read_u8());
        // should not have more (e.g. length of fixed header ended here)
        assert!(!reader.has_more());
        // after another reset more of the buffer is available
        reader.reset_limit();
        assert!(reader.has_more());
        assert_eq!(Ok(64), reader.read_u8());
        assert_eq!(Ok(128), reader.read_u8());
        // now we are done for real
        assert_eq!(false, reader.has_more());
    }

    #[test]
    // this is important since we might be
    fn test_multiple_limits_discard() {
        let src = Cursor::new(vec![0, 4, 8, 16, 32, 64, 128]);
        let r = BufReader::new(src);
        let mut reader = ByteReader::new(r);
        reader.take(5);
        assert_eq!(Ok(0), reader.read_u8());
        // take less now, 4 were left
        reader.take(3);
        assert!(reader.has_more());
        assert_eq!(Ok(4), reader.read_u8());
        assert!(reader.has_more());
        assert!(reader.consume().is_ok());
        reader.reset_limit();
        assert_eq!(Ok(32), reader.read_u8());
        // should not have more because initial limit of 5 ends here
        assert_eq!(false, reader.has_more());
        // after another reset more of the buffer is available
        reader.reset_limit();
        assert_eq!(true, reader.has_more());
        assert_eq!(Ok(64), reader.read_u8());
        assert_eq!(true, reader.has_more());
        assert_eq!(Ok(128), reader.read_u8());
        // now we are done for real
        assert_eq!(false, reader.has_more());
    }
}
