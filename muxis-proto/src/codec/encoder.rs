use bytes::{BufMut, Bytes, BytesMut};

use crate::frame::Frame;

/// A RESP encoder that converts [`Frame`] types to bytes.
///
/// The encoder accumulates data in an internal buffer and can be used
/// to encode multiple frames sequentially.
///
/// # Example
///
/// ```
/// use muxis_proto::codec::Encoder;
/// use muxis_proto::frame::Frame;
///
/// let mut encoder = Encoder::new();
/// encoder.encode(&Frame::SimpleString(b"OK".to_vec()));
/// let data = encoder.finish();
/// assert!(!data.is_empty());
/// ```
pub struct Encoder {
    buf: BytesMut,
}

impl Encoder {
    /// Creates a new encoder with an empty buffer.
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

    /// Encodes a frame into the internal buffer using RESP protocol.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to encode
    pub fn encode(&mut self, frame: &Frame) {
        match frame {
            Frame::SimpleString(s) => {
                self.buf.put_u8(b'+');
                self.buf.extend_from_slice(s);
                self.buf.extend_from_slice(b"\r\n");
            }
            Frame::Error(e) => {
                self.buf.put_u8(b'-');
                self.buf.extend_from_slice(e);
                self.buf.extend_from_slice(b"\r\n");
            }
            Frame::Integer(n) => {
                self.buf.put_u8(b':');
                self.buf.extend_from_slice(n.to_string().as_bytes());
                self.buf.extend_from_slice(b"\r\n");
            }
            Frame::BulkString(s) => {
                self.buf.put_u8(b'$');
                if let Some(data) = s {
                    self.buf
                        .extend_from_slice(data.len().to_string().as_bytes());
                    self.buf.extend_from_slice(b"\r\n");
                    self.buf.extend_from_slice(data);
                } else {
                    self.buf.extend_from_slice(b"-1");
                }
                self.buf.extend_from_slice(b"\r\n");
            }
            Frame::Array(a) => {
                self.buf.put_u8(b'*');
                self.buf.extend_from_slice(a.len().to_string().as_bytes());
                self.buf.extend_from_slice(b"\r\n");
                for item in a {
                    self.encode(item);
                }
            }
            Frame::Null => {
                self.buf.extend_from_slice(b"$-1\r\n");
            }
        }
    }

    /// Consumes the encoder and returns the encoded data.
    ///
    /// # Returns
    ///
    /// The accumulated bytes as a [`BytesMut`]
    pub fn finish(self) -> BytesMut {
        self.buf
    }

    /// Takes the encoded data from the buffer, leaving it empty.
    ///
    /// Unlike [`finish`](Encoder::finish), this method allows reusing the encoder.
    ///
    /// # Returns
    ///
    /// The accumulated bytes
    pub fn take(&mut self) -> BytesMut {
        std::mem::replace(&mut self.buf, BytesMut::new())
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Encodes a frame and returns the result as bytes.
///
/// This is a convenience function for encoding a single frame.
///
/// # Arguments
///
/// * `frame` - The frame to encode
///
/// # Returns
///
/// The encoded bytes
pub fn encode_frame(frame: &Frame) -> Bytes {
    let mut encoder = Encoder::new();
    encoder.encode(frame);
    encoder.finish().freeze()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple_string() {
        let mut encoder = Encoder::new();
        encoder.encode(&Frame::SimpleString(b"OK".to_vec()));
        assert_eq!(encoder.finish().freeze().as_ref(), b"+OK\r\n");
    }

    #[test]
    fn test_encode_error() {
        let mut encoder = Encoder::new();
        encoder.encode(&Frame::Error(b"ERR".to_vec()));
        assert_eq!(encoder.finish().freeze().as_ref(), b"-ERR\r\n");
    }

    #[test]
    fn test_encode_integer() {
        let mut encoder = Encoder::new();
        encoder.encode(&Frame::Integer(42));
        assert_eq!(encoder.finish().freeze().as_ref(), b":42\r\n");
    }

    #[test]
    fn test_encode_bulk_string() {
        let mut encoder = Encoder::new();
        encoder.encode(&Frame::BulkString(Some(Bytes::from("hello"))));
        assert_eq!(encoder.finish().freeze().as_ref(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_encode_bulk_string_null() {
        let mut encoder = Encoder::new();
        encoder.encode(&Frame::BulkString(None));
        assert_eq!(encoder.finish().freeze().as_ref(), b"$-1\r\n");
    }

    #[test]
    fn test_encode_array() {
        let mut encoder = Encoder::new();
        encoder.encode(&Frame::Array(vec![
            Frame::BulkString(Some(Bytes::from("foo"))),
            Frame::BulkString(Some(Bytes::from("bar"))),
        ]));
        assert_eq!(
            encoder.finish().freeze().as_ref(),
            b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"
        );
    }

    #[test]
    fn test_encode_null() {
        let mut encoder = Encoder::new();
        encoder.encode(&Frame::Null);
        assert_eq!(encoder.finish().freeze().as_ref(), b"$-1\r\n");
    }
}
