use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use muxis_proto::codec::{Decoder, Encoder};
use muxis_proto::frame::Frame;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};

/// A connection to a Redis server.
///
/// This struct wraps an underlying stream (TCP, TLS, etc.) and handles
/// RESP frame encoding and decoding. It implements [`AsyncRead`] and [`AsyncWrite`]
/// for composability with other async I/O types.
///
/// # Example
///
/// ```ignore
/// use muxis_core::Connection;
/// use tokio::net::TcpStream;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let stream = TcpStream::connect("127.0.0.1:6379").await?;
///     let mut conn = Connection::new(stream);
///
///     // Write a PING command
///     use muxis_proto::Frame;
///     let cmd = Frame::Array(vec![Frame::BulkString(Some("PING".into()))]);
///     conn.write_frame(&cmd).await?;
///
///     // Read the PONG response
///     let resp = conn.read_frame().await?;
///     println!("{:?}", resp);
///
///     Ok(())
/// }
/// ```
pub struct Connection<S> {
    stream: S,
    decoder: Decoder,
    encoder: Encoder,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a new connection with the given stream.
    ///
    /// Initializes a connection with no timeouts configured.
    ///
    /// # Arguments
    ///
    /// * `stream` - The underlying async stream (TCP, TLS, etc.)
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            decoder: Decoder::new(),
            encoder: Encoder::new(),
            read_timeout: None,
            write_timeout: None,
        }
    }

    /// Configures read and write timeouts for this connection.
    ///
    /// # Arguments
    ///
    /// * `read_timeout` - Optional duration for read operations
    /// * `write_timeout` - Optional duration for write operations
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_timeouts(
        mut self,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    ) -> Self {
        self.read_timeout = read_timeout;
        self.write_timeout = write_timeout;
        self
    }

    /// Writes a frame to the connection.
    ///
    /// Encodes the frame using RESP protocol and sends it to the server.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to send
    ///
    /// # Errors
    ///
    /// Returns an error if encoding or writing fails
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), std::io::Error> {
        self.encoder.encode(frame);
        let data = self.encoder.take();
        self.stream.write_all(&data).await?;
        Ok(())
    }

    /// Reads a frame from the connection.
    ///
    /// Waits for incoming data, decodes RESP frames, and returns a complete frame.
    /// This method handles partial reads internally and will continue reading
    /// until a complete frame is received or an error occurs.
    ///
    /// # Returns
    ///
    /// The decoded frame on success
    ///
    /// # Errors
    ///
    /// Returns an error if the connection is closed or a protocol error occurs
    pub async fn read_frame(&mut self) -> Result<Frame, crate::Error> {
        loop {
            match self.decoder.decode() {
                Ok(Some(frame)) => return Ok(frame),
                Ok(None) => {
                    let mut buf = vec![0u8; 4096];
                    let n = self.stream.read(&mut buf).await.map_err(|e| {
                        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                    })?;
                    if n == 0 {
                        return Err(crate::Error::Protocol {
                            message: "connection closed".to_string(),
                        });
                    }
                    self.decoder.append(&buf[..n]);
                }
                Err(e) => {
                    return Err(crate::Error::Protocol { message: e });
                }
            }
        }
    }
}

impl<S> fmt::Debug for Connection<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connection")
            .field("stream", &self.stream)
            .field("read_timeout", &self.read_timeout)
            .field("write_timeout", &self.write_timeout)
            .finish()
    }
}

impl<S> AsyncRead for Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        match Pin::new(&mut this.stream).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                if !buf.filled().is_empty() {
                    this.decoder.append(buf.filled());
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S> AsyncWrite for Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::Barrier;

    #[tokio::test]
    async fn test_connection_ping_pong() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let barrier = Arc::new(Barrier::new(2));

        let barrier_cloned = barrier.clone();
        let server = async move {
            barrier_cloned.wait().await;
            let (stream, _) = listener.accept().await.unwrap();
            let mut conn = Connection::new(stream);
            let frame = conn.read_frame().await.unwrap();
            assert_eq!(
                frame,
                Frame::Array(vec![Frame::BulkString(Some("PING".into()))])
            );
            conn.write_frame(&Frame::SimpleString(b"PONG".to_vec()))
                .await
                .unwrap();
        };

        let client = async {
            barrier.wait().await;
            let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
            let mut conn = Connection::new(stream);
            conn.write_frame(&Frame::Array(vec![Frame::BulkString(Some("PING".into()))]))
                .await
                .unwrap();
            let frame = conn.read_frame().await.unwrap();
            assert_eq!(frame, Frame::SimpleString(b"PONG".to_vec()));
        };

        tokio::join!(server, client);
    }
}
