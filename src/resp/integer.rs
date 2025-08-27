use super::{extrate_simple_frame_data, CRLF_LEN};
use crate::resp::{RespDecode, RespEncode, RespError, SimpleString};
use bytes::BytesMut;

impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extrate_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]).into_owned();
        Ok(s.parse::<i64>()?)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let len = extrate_simple_frame_data(buf, Self::PREFIX)?;
        Ok(len + CRLF_LEN)
    }
}
//- integer: ":[<+|->]<value>\r\n"
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self < 0 { "" } else { "+" };
        format!(":{sign}{self}\r\n").into_bytes()
    }
}
//- simple string: "+<value>\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::frame::RespFrame;
    use anyhow::Result;

    #[test]
    fn test_integer_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b":+123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 123);

        buf.extend_from_slice(b":-123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, -123);

        Ok(())
    }

    #[test]
    fn test_encode_integer() {
        let frame: RespFrame = 42.into();
        assert_eq!(frame.encode(), b":+42\r\n");
    }
}
