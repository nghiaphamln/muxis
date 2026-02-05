use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use muxis_proto::codec::{Decoder, Encoder};
use muxis_proto::frame::Frame;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

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
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            decoder: Decoder::new(),
            encoder: Encoder::new(),
            read_timeout: None,
            write_timeout: None,
        }
    }

    pub fn with_timeouts(
        mut self,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    ) -> Self {
        self.read_timeout = read_timeout;
        self.write_timeout = write_timeout;
        self
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), std::io::Error> {
        self.encoder.encode(frame);
        let data = self.encoder.take();
        let mut offset = 0;
        while offset < data.len() {
            match Pin::new(&mut self.stream).poll_write(
                &mut Context::from_waker(futures::task::noop_waker_ref()),
                &data[offset..],
            ) {
                Poll::Ready(Ok(n)) => {
                    if n == 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::WriteZero,
                            "failed to write",
                        ));
                    }
                    offset += n;
                }
                Poll::Ready(Err(e)) => return Err(e),
                Poll::Pending => {
                    if let Some(timeout) = self.write_timeout {
                        let _ =
                            tokio::time::timeout(timeout, futures::future::pending::<()>()).await;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn read_frame(&mut self) -> Result<Frame, crate::Error> {
        loop {
            if let Some(frame) = self
                .decoder
                .decode()
                .map_err(|e| crate::Error::Protocol { message: e })?
            {
                return Ok(frame);
            }

            if !self.decoder.has_decodable_frame() {
                return Err(crate::Error::Protocol {
                    message: "invalid frame".to_string(),
                });
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
                this.decoder.append(buf.filled());
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
    use tokio::net::TcpListener;

    #[tokio::test]
    #[ignore]
    async fn test_connection_ping_pong() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = async {
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
            let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
            let mut conn = Connection::new(stream);
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            conn.write_frame(&Frame::Array(vec![Frame::BulkString(Some("PING".into()))]))
                .await
                .unwrap();
            let frame = conn.read_frame().await.unwrap();
            assert_eq!(frame, Frame::SimpleString(b"PONG".to_vec()));
        };

        tokio::join!(server, client);
    }
}
