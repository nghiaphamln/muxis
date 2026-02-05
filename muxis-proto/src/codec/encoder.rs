use bytes::{BufMut, Bytes, BytesMut};

use crate::frame::Frame;

pub struct Encoder {
    buf: BytesMut,
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            buf: BytesMut::new(),
        }
    }

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

    pub fn finish(self) -> BytesMut {
        self.buf
    }

    pub fn take(&mut self) -> BytesMut {
        std::mem::replace(&mut self.buf, BytesMut::new())
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

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
