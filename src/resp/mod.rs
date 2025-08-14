mod array;
mod bool;
mod bulk_string;
mod double;
mod frame;
mod integer;
mod map;
mod null;
mod set;
mod simple_error;
mod simple_string;

use crate::resp::frame::{RespDecode, RespError, RespFrame, SimpleString};
use bytes::{Buf, BytesMut};

const BUF_CAP: usize = 16;
const CRLF_LEN: usize = 2; // \r\n

pub fn calc_total_length(
    buf: &[u8],
    end: usize,
    len: usize,
    prefix: &str,
) -> Result<usize, RespError> {
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];
    match prefix {
        "*" | "~" => {
            // find nth CRLF in the buffer, for array and set, we need to find 1 CRLF for each element
            for _ in 0..len {
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            // find nth CRLF in the buffer. For map, we need to find 2 CRLF for each key-value pair
            for _ in 0..len {
                let len = SimpleString::expect_length(data)?;

                data = &data[len..];
                total += len;

                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}

pub fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
    let end = extrate_simple_frame_data(buf, prefix)?;
    let len_str = &buf[prefix.len()..end];
    let len = String::from_utf8_lossy(len_str).parse::<usize>()?;
    Ok((end, len))
}

// utility functions
pub fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: {expect_type}, got: {buf:?}"
        )));
    }

    buf.advance(expect.len());
    Ok(())
}

pub fn extrate_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    if buf.len() < prefix.len() + 2 {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: SimpleString({prefix:?}), got: {buf:?}"
        )));
    }

    let end = find_clrf(buf, 1).ok_or(RespError::NotComplete)?;
    Ok(end)
}

pub fn find_clrf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }
    None
}
