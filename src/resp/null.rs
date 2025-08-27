use super::{calc_total_length, extractt_fixed_data, parse_length};
use crate::resp::{RespDecode, RespEncode, RespError};
use bytes::BytesMut;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extractt_fixed_data(buf, "_\r\n", "Null")?;
        Ok(RespNull)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}
impl Deref for RespNull {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        "null"
    }
}
// -null:"_\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::frame::RespFrame;
    use anyhow::Result;

    #[test]
    fn test_null_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"_\r\n");

        let frame = RespNull::decode(&mut buf)?;
        assert_eq!(frame, RespNull);

        Ok(())
    }

    #[test]
    fn test_encode_null() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }
}
