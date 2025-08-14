use super::{extrate_simple_frame_data, CRLF_LEN};
use crate::resp::frame::{RespDecode, RespError, SimpleString};
use bytes::BytesMut;
use std::ops::Deref;

impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extrate_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + 2); // +2 for CRLF
        let s = String::from_utf8_lossy(&data[1..end]).into_owned();
        Ok(SimpleString(s))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extrate_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}
impl Deref for SimpleString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::frame::{RespEncode, RespFrame, SimpleString};
    use anyhow::Result;
    use bytes::BufMut;

    #[test]
    fn test_simple_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");

        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString("OK".to_string()));

        buf.extend_from_slice(b"+hello\r");

        let ret = SimpleString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString("hello".to_string()));

        Ok(())
    }

    #[test]
    fn test_encode_simple_string() {
        let frame: RespFrame = SimpleString("hello".to_string()).into();
        assert_eq!(frame.encode(), b"+hello\r\n");
    }
}
