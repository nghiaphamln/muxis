use bytes::Bytes;
use muxis_proto::frame::Frame;

pub struct Cmd {
    args: Vec<Bytes>,
}

impl Cmd {
    pub fn new(name: impl Into<Bytes>) -> Self {
        Self {
            args: vec![name.into()],
        }
    }

    pub fn arg<T: Into<Bytes>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn into_frame(self) -> Frame {
        Frame::Array(
            self.args
                .into_iter()
                .map(|b| Frame::BulkString(Some(b)))
                .collect(),
        )
    }
}

pub fn ping() -> Cmd {
    Cmd::new("PING")
}

pub fn echo(msg: impl Into<Bytes>) -> Cmd {
    Cmd::new("ECHO").arg(msg)
}

pub fn get(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("GET").arg(key)
}

pub fn set(key: impl Into<Bytes>, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("SET").arg(key).arg(value)
}

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

pub fn del(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("DEL").arg(key)
}

pub fn incr(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("INCR").arg(key)
}

pub fn incr_by(key: impl Into<Bytes>, amount: i64) -> Cmd {
    Cmd::new("INCRBY").arg(key).arg(amount.to_string())
}

pub fn decr(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("DECR").arg(key)
}

pub fn decr_by(key: impl Into<Bytes>, amount: i64) -> Cmd {
    Cmd::new("DECRBY").arg(key).arg(amount.to_string())
}

pub fn auth(password: impl Into<Bytes>) -> Cmd {
    Cmd::new("AUTH").arg(password)
}

pub fn auth_with_username(username: impl Into<Bytes>, password: impl Into<Bytes>) -> Cmd {
    Cmd::new("AUTH").arg(username).arg(password)
}

pub fn select(db: u8) -> Cmd {
    Cmd::new("SELECT").arg(db.to_string())
}

pub fn client_setname(name: impl Into<Bytes>) -> Cmd {
    Cmd::new("CLIENT").arg("SETNAME").arg(name)
}

pub fn parse_frame_response(frame: Frame) -> Result<Frame, crate::Error> {
    match frame {
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Ok(frame),
    }
}

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
