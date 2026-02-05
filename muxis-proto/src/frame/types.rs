use bytes::Bytes;

/// A RESP (Redis Serialization Protocol) frame.
///
/// This enum represents all frame types defined in the RESP protocol:
/// - SimpleString: Status responses like "OK"
/// - Error: Error responses from the server
/// - Integer: Numeric responses
/// - BulkString: Binary-safe string data
/// - Array: Command arguments and array responses
/// - Null: NULL value
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
    /// Converts the frame to a human-readable string representation.
    ///
    /// For complex frames like arrays, returns a formatted string representation.
    ///
    /// # Returns
    ///
    /// Some(String) if conversion succeeds, None for frames without string representation
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

    /// Attempts to extract a bulk string from this frame.
    ///
    /// # Returns
    ///
    /// Some(Bytes) if this is a BulkString, None otherwise
    pub fn to_bulk_string(&self) -> Option<Bytes> {
        match self {
            Frame::BulkString(b) => b.clone(),
            _ => None,
        }
    }

    /// Attempts to extract an array from this frame.
    ///
    /// # Returns
    ///
    /// Some(`Vec<Frame>`) if this is an Array, None otherwise
    pub fn to_array(&self) -> Option<Vec<Frame>> {
        match self {
            Frame::Array(a) => Some(a.clone()),
            _ => None,
        }
    }

    /// Attempts to extract an integer from this frame.
    ///
    /// # Returns
    ///
    /// Some(i64) if this is an Integer, None otherwise
    pub fn to_int(&self) -> Option<i64> {
        match self {
            Frame::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns true if this frame is Null.
    pub fn is_null(&self) -> bool {
        matches!(self, Frame::Null)
    }
}
