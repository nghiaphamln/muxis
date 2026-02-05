use bytes::Bytes;

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    SimpleString(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    BulkString(Option<Bytes>),
    Array(Vec<Frame>),
    Null,
}

impl Frame {
    pub fn to_string(&self) -> Option<String> {
        match self {
            Frame::SimpleString(s) => String::from_utf8(s.clone()).ok(),
            Frame::Error(e) => String::from_utf8(e.clone()).ok(),
            Frame::Integer(i) => Some(i.to_string()),
            Frame::BulkString(b) => b.as_ref().map(|s| String::from_utf8_lossy(s).into_owned()),
            Frame::Array(a) => Some(format!(
                "[{}]",
                a.iter()
                    .filter_map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Frame::Null => Some("nil".to_string()),
        }
    }

    pub fn to_bulk_string(&self) -> Option<Bytes> {
        match self {
            Frame::BulkString(b) => b.clone(),
            _ => None,
        }
    }

    pub fn to_array(&self) -> Option<Vec<Frame>> {
        match self {
            Frame::Array(a) => Some(a.clone()),
            _ => None,
        }
    }

    pub fn to_int(&self) -> Option<i64> {
        match self {
            Frame::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Frame::Null)
    }
}
