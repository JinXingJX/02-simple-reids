use super::extract_fixed_data;
use crate::resp::frame::{RespDecode, RespEncode, RespError};
use bytes::BytesMut;

impl RespDecode for bool {
    const PREFIX: &'static str = "#";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "#t\r\n", "Bool") {
            Ok(_) => Ok(true),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", "Bool") {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(4) // "#t\r\n" or "#f\r\n"
    }
}
// -boolean: ":<value>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        let value = if self { "1" } else { "0" };
        format!(":{value}\r\n").into_bytes()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::frame::RespFrame;
    use anyhow::Result;
    use bytes::BufMut;

    #[test]
    fn test_boolean_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"#t\r\n");

        let frame = bool::decode(&mut buf)?;
        assert!(frame);

        buf.extend_from_slice(b"#f\r\n");

        let frame = bool::decode(&mut buf)?;
        assert!(!frame);

        buf.extend_from_slice(b"#f\r");
        let ret = bool::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame = bool::decode(&mut buf)?;
        assert!(!frame);

        Ok(())
    }

    #[test]
    fn test_encode_boolean() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b":1\r\n");
    }
}
