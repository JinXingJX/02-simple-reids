use super::{extrate_simple_frame_data, CRLF_LEN};
use crate::resp::frame::{RespDecode, RespEncode, RespError, SimpleError};
use bytes::BytesMut;
use std::ops::Deref;

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extrate_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + CRLF_LEN); // +2 for CRLF
        let s = String::from_utf8_lossy(&data[1..end]).into_owned();
        Ok(SimpleError(s))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extrate_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}
impl Deref for SimpleError {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// - error: "- Error message"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("- {}\r\n", self.0).into_bytes()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::frame::RespFrame;
    use anyhow::Result;

    #[test]
    fn test_simple_error_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-Error message\r\n");

        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError("Error message".to_string()));

        Ok(())
    }

    #[test]
    fn test_encode_error() {
        let frame: RespFrame = SimpleError("error".to_string()).into();
        assert_eq!(frame.encode(), b"- error\r\n");
    }
}
