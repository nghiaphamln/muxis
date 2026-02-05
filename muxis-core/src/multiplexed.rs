use std::fmt;
use tokio::sync::{mpsc, oneshot};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, instrument, error};
use muxis_proto::frame::Frame;
use crate::connection::{Connection, ConnectionReader, ConnectionWriter};

/// A request sent to the multiplexer.
struct Request {
    frame: Frame,
    response_tx: oneshot::Sender<crate::Result<Frame>>,
}

/// A handle to a multiplexed connection.
///
/// This handle is cheap to clone and can be shared across multiple tasks.
/// It provides a way to send commands to the Redis server concurrently.
#[derive(Clone)]
pub struct MultiplexedConnection {
    sender: mpsc::Sender<Request>,
}

impl MultiplexedConnection {
    /// Creates a new multiplexed connection.
    ///
    /// # Arguments
    ///
    /// * `connection` - The underlying connection to multiplex.
    /// * `queue_size` - The maximum number of pending requests.
    pub fn new<S>(connection: Connection<S>, queue_size: usize) -> Self
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (reader, writer) = connection.split();
        let (request_tx, request_rx) = mpsc::channel(queue_size);
        // Waiter queue matches request queue size plus a buffer for in-flight IO
        let (waiter_tx, waiter_rx) = mpsc::channel(queue_size);

        // Spawn writer task
        tokio::spawn(async move {
            run_writer(writer, request_rx, waiter_tx).await;
        });

        // Spawn reader task
        tokio::spawn(async move {
            run_reader(reader, waiter_rx).await;
        });

        Self { sender: request_tx }
    }

    /// Sends a command to the server and awaits the response.
    #[instrument(skip(self), level = "debug")]
    pub async fn send_command(&self, frame: Frame) -> crate::Result<Frame> {
        let (response_tx, response_rx) = oneshot::channel();
        let request = Request {
            frame,
            response_tx,
        };

        // Send request to writer task
        self.sender
            .send(request)
            .await
            .map_err(|_| crate::Error::Io {
                source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "connection closed"),
            })?;

        // Await response
        response_rx.await.map_err(|_| crate::Error::Io {
            source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "connection closed"),
        })?
    }
}

impl fmt::Debug for MultiplexedConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultiplexedConnection")
            .field("sender", &self.sender)
            .finish()
    }
}

async fn run_writer<S>(
    mut writer: ConnectionWriter<S>,
    mut request_rx: mpsc::Receiver<Request>,
    waiter_tx: mpsc::Sender<oneshot::Sender<crate::Result<Frame>>>,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    while let Some(req) = request_rx.recv().await {
        debug!(?req.frame, "sending frame");
        // Write frame to socket
        if let Err(e) = writer.write_frame(&req.frame).await {
            error!(error = ?e, "failed to write frame");
            // Failed to write, notify client
            let _ = req.response_tx.send(Err(crate::Error::Io { source: e }));
            return; // Stop writer task
        }

        // Send waiter to reader task
        // If this fails, it means reader task is dead
        if waiter_tx.send(req.response_tx).await.is_err() {
            return;
        }
    }
}

async fn run_reader<S>(
    mut reader: ConnectionReader<S>,
    mut waiter_rx: mpsc::Receiver<oneshot::Sender<crate::Result<Frame>>>,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        // Wait for the next expected response waiter
        let tx = match waiter_rx.recv().await {
            Some(tx) => tx,
            None => return, // Writer closed, no more requests coming
        };

        // Read the next frame from the connection
        match reader.read_frame().await {
            Ok(frame) => {
                debug!(?frame, "received frame");
                let _ = tx.send(Ok(frame));
            }
            Err(e) => {
                error!(error = ?e, "failed to read frame");
                let _ = tx.send(Err(e));
                // If we hit a protocol error or IO error, the connection is likely dead.
                // We should stop the reader.
                return;
            }
        }
    }
}
