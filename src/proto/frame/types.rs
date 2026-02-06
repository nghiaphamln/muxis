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
    /// Simple string (+OK).
    SimpleString(Vec<u8>),
    /// Error (-ERR).
    Error(Vec<u8>),
    /// Integer (:1000).
    Integer(i64),
    /// Bulk string ($6\r\nfoobar).
    BulkString(Option<Bytes>),
    /// Array (*2\r\n...).
    Array(Vec<Frame>),
    /// Null ($-1 or *-1).
    Null,
}

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_to_string() {
        let frame = Frame::SimpleString(b"OK".to_vec());
        assert_eq!(frame.to_string(), Some("OK".to_string()));

        let frame = Frame::Integer(42);
        assert_eq!(frame.to_string(), Some("42".to_string()));

        let frame = Frame::Null;
        assert_eq!(frame.to_string(), Some("nil".to_string()));
    }

    #[test]
    fn test_frame_to_bulk_string() {
        let data: Bytes = "hello".into();
        let frame = Frame::BulkString(Some(data.clone()));
        assert_eq!(frame.to_bulk_string(), Some(data));

        let frame = Frame::Integer(42);
        assert_eq!(frame.to_bulk_string(), None);
    }

    #[test]
    fn test_frame_to_array() {
        let frames = vec![Frame::Integer(1), Frame::Integer(2)];
        let frame = Frame::Array(frames.clone());
        assert_eq!(frame.to_array(), Some(frames));

        let frame = Frame::Integer(42);
        assert_eq!(frame.to_array(), None);
    }

    #[test]
    fn test_frame_to_int() {
        let frame = Frame::Integer(42);
        assert_eq!(frame.to_int(), Some(42));

        let frame = Frame::Null;
        assert_eq!(frame.to_int(), None);
    }

    #[test]
    fn test_frame_is_null() {
        assert!(Frame::Null.is_null());
        assert!(!Frame::Integer(42).is_null());
    }

    #[test]
    fn test_frame_array_to_string() {
        let frames = vec![
            Frame::Integer(1),
            Frame::SimpleString(b"test".to_vec()),
            Frame::Integer(3),
        ];
        let frame = Frame::Array(frames);
        assert_eq!(frame.to_string(), Some("[1, test, 3]".to_string()));
    }
}
