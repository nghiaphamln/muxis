use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task::JoinHandle;

use crate::command::Cmd;
use crate::connection::Connection;
use crate::Error;
use muxis_proto::error::Result;
use muxis_proto::frame::Frame;

/// Response to a command.
pub type Response = Result<Frame>;

/// Command ID for correlation between requests and responses.
type CommandId = u64;

/// Inner state shared between the multiplexed connection and the background task.
///
/// This struct holds the pending requests map and the connection, both wrapped
/// in Arc<Mutex<>> for thread-safe shared access between the main connection
/// and the background task.
struct MultiplexedState<S> {
    pending_requests: Arc<Mutex<HashMap<CommandId, oneshot::Sender<Response>>>>,
    connection: Arc<Mutex<Connection<S>>>,
}

/// Multiplexed connection that handles concurrent requests to a Redis server.
pub struct MultiplexedConnection<S> {
    sender: mpsc::UnboundedSender<(CommandId, Frame, oneshot::Sender<Response>)>,
    next_id: std::sync::atomic::AtomicU64,
    _connection_handle: JoinHandle<()>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S> MultiplexedConnection<S>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    /// Creates a new multiplexed connection.
    pub fn new(stream: S) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let connection = Arc::new(Mutex::new(Connection::new(stream)));
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));

        let state = Arc::new(Mutex::new(MultiplexedState {
            pending_requests: Arc::clone(&pending_requests),
            connection: Arc::clone(&connection),
        }));

        let state_clone = Arc::clone(&state);
        let connection_handle = tokio::spawn(async move {
            Self::handle_connection(state_clone, receiver).await;
        });

        Self {
            sender,
            next_id: std::sync::atomic::AtomicU64::new(0),
            _connection_handle: connection_handle,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Sends a command and waits for the response.
    pub async fn send_command(&mut self, cmd: Cmd) -> Result<Frame> {
        let cmd_frame = cmd.into_frame();
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let (response_tx, response_rx) = oneshot::channel();

        self.sender
            .send((id, cmd_frame, response_tx))
            .map_err(|_| Error::Io {
                source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "connection closed"),
            })?;

        response_rx.await.map_err(|_| Error::Io {
            source: std::io::Error::new(std::io::ErrorKind::BrokenPipe, "response channel closed"),
        })?
    }

    /// Background task that handles sending requests and receiving responses.
    async fn handle_connection(
        state: Arc<Mutex<MultiplexedState<S>>>,
        mut receiver: mpsc::UnboundedReceiver<(CommandId, Frame, oneshot::Sender<Response>)>,
    ) {
        loop {
            tokio::select! {
                // Handle incoming requests to send
                request = receiver.recv() => {
                    match request {
                        Some((id, frame, response_tx)) => {
                            // Store the pending request
                            {
                                let state_guard = state.lock().await;
                                state_guard.pending_requests.lock().await.insert(id, response_tx);
                            }

                            // Send the frame to the connection
                            let write_result = {
                                let state_guard = state.lock().await;
                                let mut conn_guard = state_guard.connection.lock().await;
                                conn_guard.write_frame(&frame).await
                            };

                            if let Err(e) = write_result {
                                // Remove the pending request and send error
                                let maybe_sender = {
                                    let state_guard = state.lock().await;
                                    let mut pending_guard = state_guard.pending_requests.lock().await;
                                    pending_guard.remove(&id)
                                };

                                if let Some(sender) = maybe_sender {
                                    let _ = sender.send(Err(Error::Io { source: e }));
                                }
                                continue;
                            }
                        }
                        None => break, // Channel closed, exit the loop
                    }
                }
                // Handle incoming responses
                result = async {
                    let state_guard = state.lock().await;
                    let mut conn_guard = state_guard.connection.lock().await;
                    conn_guard.read_frame().await
                } => {
                    match result {
                        Ok(frame) => {
                            // Get and remove the first pending request to respond to
                            let maybe_request_data = {
                                let state_guard = state.lock().await;
                                let mut pending_guard = state_guard.pending_requests.lock().await;
                                if let Some((request_id, _sender)) = pending_guard.iter().next() {
                                    let id = *request_id; // Copy the ID
                                    let sender_to_return = pending_guard.remove(&id);
                                    Some((id, sender_to_return))
                                } else {
                                    None
                                }
                            };

                            if let Some((_request_id, Some(sender))) = maybe_request_data {
                                let _ = sender.send(Ok(frame));
                            }
                        }
                        Err(e) => {
                            // Get all pending requests and notify them of the error
                            let senders = {
                                let state_guard = state.lock().await;
                                let mut pending_guard = state_guard.pending_requests.lock().await;
                                let mut temp_senders = HashMap::new();
                                std::mem::swap(&mut temp_senders, &mut *pending_guard);
                                temp_senders
                            };

                            // Create a new error to send to all pending requests
                            for (_id, sender) in senders {
                                let err = match &e {
                                    Error::Io { source } => Error::Io { source: std::io::Error::new(source.kind(), source.to_string()) },
                                    Error::Protocol { message } => Error::Protocol { message: message.clone() },
                                    Error::Server { message } => Error::Server { message: message.clone() },
                                    Error::Auth => Error::Auth,
                                    Error::InvalidArgument { message } => Error::InvalidArgument { message: message.clone() },
                                    Error::Encode { source } => Error::Encode { source: muxis_proto::error::EncodeError::new(std::io::Error::new(std::io::ErrorKind::Other, source.to_string())) },
                                    Error::Decode { source } => Error::Decode { source: muxis_proto::error::DecodeError::new(std::io::Error::new(std::io::ErrorKind::Other, source.to_string())) },
                                };
                                let _ = sender.send(Err(err));
                            }
                            break; // Exit the loop as the connection is broken
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_multiplexed_connection_creation() {
        // This is a basic test to ensure the structure is sound
        // A full integration test would require a mock connection
    }
}
