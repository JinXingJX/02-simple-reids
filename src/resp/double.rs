use super::{extrate_simple_frame_data, CRLF_LEN};
use crate::resp::{RespDecode, RespEncode, RespError};
use bytes::BytesMut;

// - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespDecode for f64 {
    const PREFIX: &'static str = ",";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extrate_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + 2);
        let s = String::from_utf8_lossy(&data[1..end]).into_owned();
        Ok(s.parse::<f64>()?)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extrate_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}
// - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 {
            format!(",{self:+e}\r\n")
        } else {
            let sign = if self < 0.0 { "" } else { "+" };
            format!(",{sign}{self}\r\n")
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::RespFrame;
    use anyhow::Result;
    use std::f64::consts::PI;

    #[test]
    fn test_double_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b",123.45\r\n");

        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 123.45);

        buf.extend_from_slice(b",+1.23456e-9\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 1.23456e-9);

        Ok(())
    }

    #[test]
    fn test_encode_double() {
        let frame: RespFrame = PI.into();
        assert_eq!(frame.encode(), b",+3.141592653589793\r\n");
    }
}
