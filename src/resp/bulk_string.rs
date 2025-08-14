use super::{
    calc_total_length, extract_fixed_data, extrate_simple_frame_data, parse_length, CRLF_LEN,
};
use crate::resp::frame::{BulkString, RespDecode, RespEncode, RespError, RespNullBulkString};
use bytes::{Buf, BytesMut};
use std::ops::Deref;

impl RespDecode for RespNullBulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "$-1\r\n", "NullBulkString")?;
        Ok(RespNullBulkString)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}
// - bulk string: "$<length>\r\n<value>\r\n"
impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extrate_simple_frame_data(buf, Self::PREFIX)?;
        let len_str = &buf[1..end];
        let len = String::from_utf8_lossy(len_str).parse::<usize>()?;
        let remained = &buf[end + 2..];
        if remained.len() < 2 + len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + 2); // advance past "$<len>\r\n"
        let data = buf.split_to(len + 2);
        Ok(BulkString::new(data[..len].to_vec()))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}
impl Deref for RespNullBulkString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        "null"
    }
}
impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}
// - bulk string: "$<length>\r\n<value>\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

// - null bulk string: "$-1\r\n"
impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::frame::{RespFrame, SimpleString};
    use anyhow::Result;
    #[test]
    fn test_null_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");

        let frame = RespNullBulkString::decode(&mut buf)?;
        assert_eq!(frame, RespNullBulkString);

        Ok(())
    }

    #[test]
    fn test_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        Ok(())
    }

    #[test]
    fn test_encode_null_bulk_string() {
        let frame: RespFrame = RespNullBulkString.into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_encode_bulk_string() {
        let frame: RespFrame = SimpleString("hello".to_string()).into();
        assert_eq!(frame.encode(), b"+hello\r\n");
    }
}
