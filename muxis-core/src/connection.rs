use std::fmt;
use std::time::Duration;

use muxis_proto::codec::{Decoder, Encoder};
use muxis_proto::frame::Frame;
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};

/// A connection to a Redis server.
///
/// This struct wraps an underlying stream (TCP, TLS, etc.) and handles
/// RESP frame encoding and decoding.
///
/// # Example
///
/// ```no_run
/// use muxis_core::connection::Connection;
/// use tokio::net::TcpStream;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let stream = TcpStream::connect("127.0.0.1:6379").await?;
///     let mut conn = Connection::new(stream);
///
///     // Write a PING command
///     use muxis_proto::frame::Frame;
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

/// Read half of a split connection.
pub struct ConnectionReader<S> {
    stream: ReadHalf<S>,
    decoder: Decoder,
    timeout: Option<Duration>,
}

/// Write half of a split connection.
pub struct ConnectionWriter<S> {
    stream: WriteHalf<S>,
    encoder: Encoder,
    timeout: Option<Duration>,
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a new connection with the given stream.
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
    pub fn with_timeouts(
        mut self,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    ) -> Self {
        self.read_timeout = read_timeout;
        self.write_timeout = write_timeout;
        self
    }

    /// Splits the connection into a read half and a write half.
    ///
    /// This allows independent reading and writing, which is useful for
    /// multiplexing.
    pub fn split(self) -> (ConnectionReader<S>, ConnectionWriter<S>) {
        let (read_half, write_half) = io::split(self.stream);
        (
            ConnectionReader {
                stream: read_half,
                decoder: self.decoder,
                timeout: self.read_timeout,
            },
            ConnectionWriter {
                stream: write_half,
                encoder: self.encoder,
                timeout: self.write_timeout,
            },
        )
    }

    /// Writes a frame to the connection.
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), std::io::Error> {
        self.encoder.encode(frame);
        let data = self.encoder.take();
        
        match self.write_timeout {
            Some(duration) => {
                tokio::time::timeout(duration, self.stream.write_all(&data))
                    .await
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "write timeout"))??;
            }
            None => {
                self.stream.write_all(&data).await?;
            }
        }
        Ok(())
    }

    /// Reads a frame from the connection.
    pub async fn read_frame(&mut self) -> Result<Frame, crate::Error> {
        loop {
            if let Some(frame) = self.decoder.decode().map_err(|e| crate::Error::Protocol { message: e })? {
                return Ok(frame);
            }

            let mut buf = vec![0u8; 4096];
            let read_future = self.stream.read(&mut buf);
            
            let n = match self.read_timeout {
                Some(duration) => {
                    tokio::time::timeout(duration, read_future)
                        .await
                        .map_err(|_| crate::Error::Io { source: std::io::Error::new(std::io::ErrorKind::TimedOut, "read timeout") })?
                        .map_err(|e| crate::Error::Io { source: e })?
                }
                None => read_future.await.map_err(|e| crate::Error::Io { source: e })?,
            };

            if n == 0 {
                return Err(crate::Error::Protocol {
                    message: "connection closed".to_string(),
                });
            }
            self.decoder.append(&buf[..n]);
        }
    }
}

impl<S> ConnectionReader<S>
where
    S: AsyncRead + AsyncWrite,
{
    /// Reads a frame from the connection.
    pub async fn read_frame(&mut self) -> Result<Frame, crate::Error> {
        loop {
            if let Some(frame) = self.decoder.decode().map_err(|e| crate::Error::Protocol { message: e })? {
                return Ok(frame);
            }

            let mut buf = vec![0u8; 4096];
            let read_future = self.stream.read(&mut buf);
            
            let n = match self.timeout {
                Some(duration) => {
                    tokio::time::timeout(duration, read_future)
                        .await
                        .map_err(|_| crate::Error::Io { source: std::io::Error::new(std::io::ErrorKind::TimedOut, "read timeout") })?
                        .map_err(|e| crate::Error::Io { source: e })?
                }
                None => read_future.await.map_err(|e| crate::Error::Io { source: e })?,
            };

            if n == 0 {
                return Err(crate::Error::Protocol {
                    message: "connection closed".to_string(),
                });
            }
            self.decoder.append(&buf[..n]);
        }
    }
}

impl<S> ConnectionWriter<S>
where
    S: AsyncRead + AsyncWrite,
{
    /// Writes a frame to the connection.
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), std::io::Error> {
        self.encoder.encode(frame);
        let data = self.encoder.take();
        
        match self.timeout {
            Some(duration) => {
                tokio::time::timeout(duration, self.stream.write_all(&data))
                    .await
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "write timeout"))??;
            }
            None => {
                self.stream.write_all(&data).await?;
            }
        }
        Ok(())
    }
}

impl<S> fmt::Debug for Connection<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Connection")
            .field("read_timeout", &self.read_timeout)
            .field("write_timeout", &self.write_timeout)
            .finish()
    }
}

impl<S> fmt::Debug for ConnectionReader<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionReader")
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl<S> fmt::Debug for ConnectionWriter<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionWriter")
            .field("timeout", &self.timeout)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::Barrier;

    #[tokio::test]
    async fn test_connection_split() {
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
            let conn = Connection::new(stream);
            let (mut reader, mut writer) = conn.split();

            writer.write_frame(&Frame::Array(vec![Frame::BulkString(Some("PING".into()))]))
                .await
                .unwrap();
            
            let frame = reader.read_frame().await.unwrap();
            assert_eq!(frame, Frame::SimpleString(b"PONG".to_vec()));
        };

        tokio::join!(server, client);
    }
}
