use super::{extract_args, validate_command, HSet};
use crate::{
    BulkString, CommandError, CommandExecutor, HGet, HGetAll, RespArray, RespFrame, RESP_OK,
};
use std::convert::TryFrom;

const ONE_ARGS: usize = 1;
const TWO_ARGS: usize = 2;

impl CommandExecutor for HGet {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        match backend.hget(&self.key, &self.field) {
            Some(value) => value,
            None => RespFrame::Null(crate::RespNull),
        }
    }
}

impl CommandExecutor for HSet {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        backend.hset(self.key, self.field, self.value);
        RESP_OK.clone()
    }
}

impl CommandExecutor for HGetAll {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let hmap = backend.hmap.get(&self.key);
        match hmap {
            Some(hmap) => {
                let mut arr = Vec::with_capacity(hmap.len());
                for v in hmap.iter() {
                    let key = v.key().to_owned();
                    arr.push((key, v.value().clone()));
                }
                if self.sort {
                    arr.sort_by(|a, b| a.0.cmp(&b.0));
                }

                let ret = arr
                    .into_iter()
                    .flat_map(|(k, v)| vec![BulkString::from(k).into(), v])
                    .collect::<Vec<RespFrame>>();

                RespArray::new(ret).into()
            }
            None => RespFrame::Null(crate::RespNull),
        }
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    fn try_from(array: RespArray) -> Result<Self, Self::Error> {
        validate_command(&array, &["hget"], TWO_ARGS)?;
        let mut args = extract_args(array, ONE_ARGS)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(ref key)), Some(RespFrame::BulkString(ref field))) => {
                Ok(HGet {
                    key: String::from_utf8_lossy(key).to_string(),
                    field: String::from_utf8_lossy(field).to_string(),
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "HGet command requires a string key and field".to_string(),
            )),
        }
    }
}
impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;

    fn try_from(array: RespArray) -> Result<Self, Self::Error> {
        validate_command(&array, &["hgetall"], ONE_ARGS)?;
        let mut args = extract_args(array, ONE_ARGS)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(ref key)) => Ok(HGetAll {
                key: String::from_utf8_lossy(key).to_string(),
                sort: false,
            }),
            _ => Err(CommandError::InvalidArgument(
                "HGetAll command requires a string key and bool".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;

    fn try_from(array: RespArray) -> Result<Self, Self::Error> {
        validate_command(&array, &["hset"], 3)?;
        let mut args = extract_args(array, ONE_ARGS)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (
                Some(RespFrame::BulkString(ref key)),
                Some(RespFrame::BulkString(ref field)),
                Some(value),
            ) => Ok(HSet {
                key: String::from_utf8_lossy(key).to_string(),
                field: String::from_utf8_lossy(field).to_string(),
                value,
            }),
            _ => Err(CommandError::InvalidArgument(
                "HSet command requires a string key, field and value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::RespDecode;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_hget_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HGet = frame.try_into()?;
        assert_eq!(result.key, "map");
        assert_eq!(result.field, "hello");

        Ok(())
    }

    #[test]
    fn test_hgetall_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$7\r\nhgetall\r\n$3\r\nmap\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HGetAll = frame.try_into()?;
        assert_eq!(result.key, "map");

        Ok(())
    }

    #[test]
    fn test_hset_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HSet = frame.try_into()?;
        assert_eq!(result.key, "map");
        assert_eq!(result.field, "hello");
        assert_eq!(result.value, RespFrame::BulkString(b"world".into()));

        Ok(())
    }
}
