use bytes::Buf;
use bytes::BytesMut;

use crate::proto::frame::Frame;

const DEFAULT_MAX_FRAME_SIZE: usize = 512 * 1024 * 1024; // 512 MB default

/// A RESP decoder that converts bytes to [`Frame`] types.
///
/// The decoder handles streaming input and can decode frames incrementally.
/// Call [`append`](Decoder::append) to add data, then [`decode`](Decoder::decode)
/// to parse frames. Returns `Ok(None)` when more data is needed.
///
/// # Example
///
/// ```
/// use muxis::proto::codec::Decoder;
/// use muxis::proto::frame::Frame;
///
/// let mut decoder = Decoder::new();
/// decoder.append(b"+OK\r\n");
/// let frame = decoder.decode().unwrap().unwrap();
/// assert_eq!(frame, Frame::SimpleString(b"OK".to_vec()));
/// ```
#[derive(Debug)]
pub struct Decoder {
    buf: BytesMut,
    max_frame_size: usize,
}

impl Decoder {
    /// Creates a new decoder with an empty buffer.
    pub fn new() -> Self {
        Self::with_max_frame_size(DEFAULT_MAX_FRAME_SIZE)
    }

    /// Creates a new decoder with a custom maximum frame size.
    ///
    /// # Arguments
    ///
    /// * `max_frame_size` - Maximum size in bytes for a single frame
    pub fn with_max_frame_size(max_frame_size: usize) -> Self {
        Self {
            buf: BytesMut::new(),
            max_frame_size,
        }
    }

    /// Appends raw bytes to the internal buffer.
    ///
    /// Call this method when new data arrives from the network.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes to append
    ///
    /// # Note
    ///
    /// Buffer size limits are checked during decode, not append.
    /// This allows for streaming large frames incrementally.
    pub fn append(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    /// Attempts to decode a frame from the buffer.
    ///
    /// Returns `Ok(Some(Frame))` if a complete frame was decoded.
    /// Returns `Ok(None)` if more data is needed.
    /// Returns `Err(...)` if the data is malformed.
    ///
    /// # Returns
    ///
    /// Decoded frame, None if incomplete, or error
    pub fn decode(&mut self) -> Result<Option<Frame>, String> {
        if self.buf.is_empty() {
            return Ok(None);
        }

        // Check if buffer size exceeds max allowed frame size
        if self.buf.len() > self.max_frame_size {
            return Err("Buffer size exceeded maximum frame size".to_string());
        }

        let frame = match self.buf[0] {
            b'+' => self.decode_simple_string(),
            b'-' => self.decode_error(),
            b':' => self.decode_integer(),
            b'$' => self.decode_bulk_string(),
            b'*' => self.decode_array(),
            _ => Err(format!("unknown frame type: {}", self.buf[0] as char)),
        };

        match frame {
            Ok(Some(frame)) => Ok(Some(frame)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn decode_simple_string(&mut self) -> Result<Option<Frame>, String> {
        let end = match self.find_crlf() {
            Some(end) => end,
            None => return Ok(None),
        };
        if end == 1 {
            return Ok(Some(Frame::SimpleString(Vec::new())));
        }
        let data = self.buf[1..end].to_vec();
        self.buf.advance(end + 2);
        Ok(Some(Frame::SimpleString(data)))
    }

    fn decode_error(&mut self) -> Result<Option<Frame>, String> {
        let end = match self.find_crlf() {
            Some(end) => end,
            None => return Ok(None),
        };
        let data = self.buf[1..end].to_vec();
        self.buf.advance(end + 2);
        Ok(Some(Frame::Error(data)))
    }

    fn decode_integer(&mut self) -> Result<Option<Frame>, String> {
        let end = match self.find_crlf() {
            Some(end) => end,
            None => return Ok(None),
        };
        let data = self.buf[1..end].to_vec();
        let num = String::from_utf8(data)
            .map_err(|e| e.to_string())?
            .parse::<i64>()
            .map_err(|e| e.to_string())?;
        self.buf.advance(end + 2);
        Ok(Some(Frame::Integer(num)))
    }

    fn decode_bulk_string(&mut self) -> Result<Option<Frame>, String> {
        let end = match self.find_crlf() {
            Some(end) => end,
            None => return Ok(None),
        };
        let len_str = String::from_utf8(self.buf[1..end].to_vec()).map_err(|e| e.to_string())?;
        let len: isize = len_str.parse::<isize>().map_err(|e| e.to_string())?;
        self.buf.advance(end + 2);

        if len == -1 {
            return Ok(Some(Frame::BulkString(None)));
        }

        let len = len as usize;

        // Check if the declared length exceeds our max frame size
        if len > self.max_frame_size {
            return Err("Bulk string length exceeds maximum frame size".to_string());
        }

        if self.buf.len() < len + 2 {
            return Ok(None);
        }

        let data = self.buf[..len].to_vec().into();
        self.buf.advance(len + 2);
        Ok(Some(Frame::BulkString(Some(data))))
    }

    fn decode_array(&mut self) -> Result<Option<Frame>, String> {
        let end = match self.find_crlf() {
            Some(end) => end,
            None => return Ok(None),
        };
        let len_str = String::from_utf8(self.buf[1..end].to_vec()).map_err(|e| e.to_string())?;
        let len: isize = len_str.parse::<isize>().map_err(|e| e.to_string())?;
        self.buf.advance(end + 2);

        if len == -1 {
            return Ok(Some(Frame::Null));
        }

        let len = len as usize;

        // Check if the array length is reasonable
        if len > self.max_frame_size / 16 {
            // Assume minimum 16 bytes per item
            return Err("Array length exceeds reasonable maximum".to_string());
        }

        let mut items = Vec::with_capacity(len);
        for _ in 0..len {
            match self.decode()? {
                Some(frame) => items.push(frame),
                None => return Ok(None),
            }
        }

        Ok(Some(Frame::Array(items)))
    }

    /// Searches for the next CRLF sequence in the buffer.
    ///
    /// # Returns
    ///
    /// Some(index) if found, None if not enough data
    fn find_crlf(&self) -> Option<usize> {
        if self.buf.len() < 2 {
            return None;
        }
        for i in 1..self.buf.len() {
            if self.buf[i - 1] == b'\r' && self.buf[i] == b'\n' {
                return Some(i - 1);
            }
        }
        None
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn test_decode_simple_string() {
        let mut decoder = Decoder::new();
        decoder.append(b"+OK\r\n");
        let frame = decoder.decode().unwrap().unwrap();
        assert_eq!(frame, Frame::SimpleString(b"OK".to_vec()));
    }

    #[test]
    fn test_decode_error() {
        let mut decoder = Decoder::new();
        decoder.append(b"-ERR some error\r\n");
        let frame = decoder.decode().unwrap().unwrap();
        assert_eq!(frame, Frame::Error(b"ERR some error".to_vec()));
    }

    #[test]
    fn test_decode_integer() {
        let mut decoder = Decoder::new();
        decoder.append(b":42\r\n");
        let frame = decoder.decode().unwrap().unwrap();
        assert_eq!(frame, Frame::Integer(42));
    }

    #[test]
    fn test_decode_bulk_string() {
        let mut decoder = Decoder::new();
        decoder.append(b"$5\r\nhello\r\n");
        let frame = decoder.decode().unwrap().unwrap();
        assert_eq!(frame, Frame::BulkString(Some(Bytes::from("hello"))));
    }

    #[test]
    fn test_decode_bulk_string_null() {
        let mut decoder = Decoder::new();
        decoder.append(b"$-1\r\n");
        let frame = decoder.decode().unwrap().unwrap();
        assert_eq!(frame, Frame::BulkString(None));
    }

    #[test]
    fn test_decode_array() {
        let mut decoder = Decoder::new();
        decoder.append(b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
        let frame = decoder.decode().unwrap().unwrap();
        assert_eq!(
            frame,
            Frame::Array(vec![
                Frame::BulkString(Some(Bytes::from("foo"))),
                Frame::BulkString(Some(Bytes::from("bar"))),
            ])
        );
    }

    #[test]
    fn test_decode_null() {
        let mut decoder = Decoder::new();
        decoder.append(b"*-1\r\n");
        let frame = decoder.decode().unwrap().unwrap();
        assert_eq!(frame, Frame::Null);
    }

    #[test]
    fn test_decode_partial() {
        let mut decoder = Decoder::new();
        decoder.append(b"+OK\r");
        assert!(decoder.decode().unwrap().is_none());
        decoder.append(b"\n");
        let frame = decoder.decode().unwrap().unwrap();
        assert_eq!(frame, Frame::SimpleString(b"OK".to_vec()));
    }

    #[test]
    fn test_decoder_with_max_frame_size() {
        let decoder = Decoder::with_max_frame_size(1024);
        assert_eq!(decoder.max_frame_size, 1024);
    }

    #[test]
    fn test_decoder_bulk_string_exceeds_max_size() {
        let mut decoder = Decoder::with_max_frame_size(10);
        // Try to decode a bulk string of 100 bytes (exceeds max of 10)
        decoder.append(b"$100\r\n");
        let result = decoder.decode();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Bulk string length exceeds maximum"));
    }

    #[test]
    fn test_decoder_array_exceeds_reasonable_max() {
        let mut decoder = Decoder::with_max_frame_size(1024);
        // Try to decode an array with way too many elements
        let huge_count = (1024 / 16) + 100; // Exceeds reasonable limit
        let data = format!("*{}\r\n", huge_count);
        decoder.append(data.as_bytes());
        let result = decoder.decode();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Array length exceeds reasonable maximum"));
    }

    #[test]
    fn test_decoder_buffer_exceeds_max_on_decode() {
        let mut decoder = Decoder::with_max_frame_size(10);
        // Append 20 bytes of data (exceeds max of 10)
        decoder.append(b"+");
        let large_data = vec![b'x'; 20];
        decoder.append(&large_data);
        decoder.append(b"\r\n");
        // Should detect overflow during decode
        let result = decoder.decode();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Buffer size exceeded maximum"));
    }
}
