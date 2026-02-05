use bytes::Bytes;
use muxis_proto::frame::Frame;

/// A command ready to be sent to Redis.
///
/// Commands are built using the builder pattern and converted to frames
/// for transmission over the connection.
///
/// # Example
///
/// ```
/// use muxis_core::command::{Cmd, get, set};
///
/// let cmd = Cmd::new("SET").arg("key").arg("value");
/// let get_cmd = get("key");
/// let set_cmd = set("key", "new_value");
/// ```
#[derive(Debug)]
pub struct Cmd {
    args: Vec<Bytes>,
}

impl Cmd {
    /// Creates a new command with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - The command name (e.g., "GET", "SET", "DEL")
    #[inline]
    pub fn new(name: impl Into<Bytes>) -> Self {
        Self {
            args: vec![name.into()],
        }
    }

    /// Appends an argument to the command.
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument value
    #[inline]
    pub fn arg<T: Into<Bytes>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Converts the command to a RESP Array frame.
    #[inline]
    pub fn into_frame(self) -> Frame {
        Frame::Array(
            self.args
                .into_iter()
                .map(|b| Frame::BulkString(Some(b)))
                .collect(),
        )
    }
}

/// Creates a PING command.
#[inline]
pub fn ping() -> Cmd {
    Cmd::new("PING")
}

/// Creates an ECHO command.
#[inline]
pub fn echo(msg: impl Into<Bytes>) -> Cmd {
    Cmd::new("ECHO").arg(msg)
}

/// Creates a GET command.
#[inline]
pub fn get(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("GET").arg(key)
}

/// Creates a SET command.
#[inline]
pub fn set(key: impl Into<Bytes>, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("SET").arg(key).arg(value)
}

/// Creates a SET command with expiration.
///
/// # Arguments
///
/// * `key` - The key to set
/// * `value` - The value to set
/// * `expiry` - Time until the key expires
#[inline]
pub fn set_with_expiry(
    key: impl Into<Bytes>,
    value: impl Into<Bytes>,
    expiry: std::time::Duration,
) -> Cmd {
    Cmd::new("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(expiry.as_secs().to_string())
}

/// Creates a DEL command.
#[inline]
pub fn del(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("DEL").arg(key)
}

/// Creates an INCR command.
#[inline]
pub fn incr(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("INCR").arg(key)
}

/// Creates an INCRBY command.
#[inline]
pub fn incr_by(key: impl Into<Bytes>, amount: i64) -> Cmd {
    Cmd::new("INCRBY").arg(key).arg(amount.to_string())
}

/// Creates a DECR command.
#[inline]
pub fn decr(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("DECR").arg(key)
}

/// Creates a DECRBY command.
#[inline]
pub fn decr_by(key: impl Into<Bytes>, amount: i64) -> Cmd {
    Cmd::new("DECRBY").arg(key).arg(amount.to_string())
}

/// Creates an AUTH command with password only.
#[inline]
pub fn auth(password: impl Into<Bytes>) -> Cmd {
    Cmd::new("AUTH").arg(password)
}

/// Creates an AUTH command with username and password (ACL style).
#[inline]
pub fn auth_with_username(username: impl Into<Bytes>, password: impl Into<Bytes>) -> Cmd {
    Cmd::new("AUTH").arg(username).arg(password)
}

/// Creates a SELECT command.
#[inline]
pub fn select(db: u8) -> Cmd {
    Cmd::new("SELECT").arg(db.to_string())
}

/// Creates a CLIENT SETNAME command.
#[inline]
pub fn client_setname(name: impl Into<Bytes>) -> Cmd {
    Cmd::new("CLIENT").arg("SETNAME").arg(name)
}

/// Parses a frame as a Redis response.
#[inline]
pub fn parse_frame_response(frame: Frame) -> Result<Frame, crate::Error> {
    match frame {
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Ok(frame),
    }
}

/// Converts a frame to bytes.
#[inline]
pub fn frame_to_bytes(frame: Frame) -> Result<Option<Bytes>, crate::Error> {
    match frame {
        Frame::BulkString(b) => Ok(b),
        Frame::Null => Ok(None),
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "unexpected frame type".to_string(),
        }),
    }
}

/// Converts a frame to an integer.
#[inline]
pub fn frame_to_int(frame: Frame) -> Result<i64, crate::Error> {
    match frame {
        Frame::Integer(i) => Ok(i),
        Frame::BulkString(b) => {
            let s = b
                .as_ref()
                .map_or("", |bytes| std::str::from_utf8(bytes).unwrap_or(""));
            s.parse::<i64>().map_err(|_| crate::Error::Protocol {
                message: "invalid integer".to_string(),
            })
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "unexpected frame type".to_string(),
        }),
    }
}

/// Converts a frame to a boolean.
#[inline]
pub fn frame_to_bool(frame: Frame) -> Result<bool, crate::Error> {
    match frame {
        Frame::Integer(i) => Ok(i != 0),
        Frame::BulkString(b) => Ok(b.map_or(false, |bytes| !bytes.is_empty())),
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "unexpected frame type".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_cmd() {
        let cmd = ping();
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![Frame::BulkString(Some("PING".into()))])
        );
    }

    #[test]
    fn test_echo_cmd() {
        let cmd = echo("hello");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ECHO".into())),
                Frame::BulkString(Some("hello".into()))
            ])
        );
    }

    #[test]
    fn test_get_cmd() {
        let cmd = get("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("GET".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_set_cmd() {
        let cmd = set("key", "value");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SET".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("value".into()))
            ])
        );
    }

    #[test]
    fn test_incr_cmd() {
        let cmd = incr("counter");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("INCR".into())),
                Frame::BulkString(Some("counter".into()))
            ])
        );
    }

    #[test]
    fn test_auth_cmd() {
        let cmd = auth("password");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("AUTH".into())),
                Frame::BulkString(Some("password".into()))
            ])
        );
    }

    #[test]
    fn test_auth_with_username() {
        let cmd = auth_with_username("user", "password");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("AUTH".into())),
                Frame::BulkString(Some("user".into())),
                Frame::BulkString(Some("password".into()))
            ])
        );
    }
}
