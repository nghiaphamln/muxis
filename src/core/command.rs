use crate::proto::frame::Frame;
use bytes::Bytes;

/// A command ready to be sent to Redis.
///
/// Commands are built using the builder pattern and converted to frames
/// for transmission over the connection.
///
/// # Example
///
/// ```
/// use muxis::core::command::{Cmd, get, set};
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

/// Creates a MGET command.
#[inline]
pub fn mget(keys: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("MGET");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd
}

/// Creates a MSET command.
#[inline]
pub fn mset(pairs: Vec<(String, Bytes)>) -> Cmd {
    let mut cmd = Cmd::new("MSET");
    for (key, value) in pairs {
        cmd = cmd.arg(key).arg(value);
    }
    cmd
}

/// Creates a SETNX command.
#[inline]
pub fn setnx(key: impl Into<Bytes>, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("SETNX").arg(key).arg(value)
}

/// Creates a SETEX command.
#[inline]
pub fn setex(key: impl Into<Bytes>, seconds: u64, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("SETEX")
        .arg(key)
        .arg(seconds.to_string())
        .arg(value)
}

/// Creates a GETDEL command.
#[inline]
pub fn getdel(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("GETDEL").arg(key)
}

/// Creates an APPEND command.
#[inline]
pub fn append(key: impl Into<Bytes>, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("APPEND").arg(key).arg(value)
}

/// Creates a STRLEN command.
#[inline]
pub fn strlen(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("STRLEN").arg(key)
}

/// Creates an EXISTS command.
#[inline]
pub fn exists(keys: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("EXISTS");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd
}

/// Creates a TYPE command.
#[inline]
pub fn key_type(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("TYPE").arg(key)
}

/// Creates an EXPIRE command.
#[inline]
pub fn expire(key: impl Into<Bytes>, seconds: u64) -> Cmd {
    Cmd::new("EXPIRE").arg(key).arg(seconds.to_string())
}

/// Creates an EXPIREAT command.
#[inline]
pub fn expireat(key: impl Into<Bytes>, timestamp: u64) -> Cmd {
    Cmd::new("EXPIREAT").arg(key).arg(timestamp.to_string())
}

/// Creates a TTL command.
#[inline]
pub fn ttl(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("TTL").arg(key)
}

/// Creates a PERSIST command.
#[inline]
pub fn persist(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("PERSIST").arg(key)
}

/// Creates a RENAME command.
#[inline]
pub fn rename(key: impl Into<Bytes>, newkey: impl Into<Bytes>) -> Cmd {
    Cmd::new("RENAME").arg(key).arg(newkey)
}

/// Creates a SCAN command.
#[inline]
pub fn scan(cursor: u64) -> Cmd {
    Cmd::new("SCAN").arg(cursor.to_string())
}

/// Creates an HSET command.
#[inline]
pub fn hset(key: impl Into<Bytes>, field: impl Into<Bytes>, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("HSET").arg(key).arg(field).arg(value)
}

/// Creates an HGET command.
#[inline]
pub fn hget(key: impl Into<Bytes>, field: impl Into<Bytes>) -> Cmd {
    Cmd::new("HGET").arg(key).arg(field)
}

/// Creates an HMSET command.
#[inline]
pub fn hmset(key: String, fields: Vec<(String, Bytes)>) -> Cmd {
    let mut cmd = Cmd::new("HMSET").arg(key);
    for (field, value) in fields {
        cmd = cmd.arg(field).arg(value);
    }
    cmd
}

/// Creates an HMGET command.
#[inline]
pub fn hmget(key: String, fields: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("HMGET").arg(key);
    for field in fields {
        cmd = cmd.arg(field);
    }
    cmd
}

/// Creates an HGETALL command.
#[inline]
pub fn hgetall(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("HGETALL").arg(key)
}

/// Creates an HDEL command.
#[inline]
pub fn hdel(key: String, fields: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("HDEL").arg(key);
    for field in fields {
        cmd = cmd.arg(field);
    }
    cmd
}

/// Creates an HEXISTS command.
#[inline]
pub fn hexists(key: impl Into<Bytes>, field: impl Into<Bytes>) -> Cmd {
    Cmd::new("HEXISTS").arg(key).arg(field)
}

/// Creates an HLEN command.
#[inline]
pub fn hlen(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("HLEN").arg(key)
}

/// Creates an HKEYS command.
#[inline]
pub fn hkeys(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("HKEYS").arg(key)
}

/// Creates an HVALS command.
#[inline]
pub fn hvals(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("HVALS").arg(key)
}

/// Creates an HINCRBY command.
#[inline]
pub fn hincrby(key: impl Into<Bytes>, field: impl Into<Bytes>, increment: i64) -> Cmd {
    Cmd::new("HINCRBY")
        .arg(key)
        .arg(field)
        .arg(increment.to_string())
}

/// Creates an HINCRBYFLOAT command.
#[inline]
pub fn hincrbyfloat(key: impl Into<Bytes>, field: impl Into<Bytes>, increment: f64) -> Cmd {
    Cmd::new("HINCRBYFLOAT")
        .arg(key)
        .arg(field)
        .arg(increment.to_string())
}

/// Creates an HSETNX command.
#[inline]
pub fn hsetnx(key: impl Into<Bytes>, field: impl Into<Bytes>, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("HSETNX").arg(key).arg(field).arg(value)
}

/// Creates an LPUSH command.
#[inline]
pub fn lpush(key: String, values: Vec<Bytes>) -> Cmd {
    let mut cmd = Cmd::new("LPUSH").arg(key);
    for value in values {
        cmd = cmd.arg(value);
    }
    cmd
}

/// Creates an RPUSH command.
#[inline]
pub fn rpush(key: String, values: Vec<Bytes>) -> Cmd {
    let mut cmd = Cmd::new("RPUSH").arg(key);
    for value in values {
        cmd = cmd.arg(value);
    }
    cmd
}

/// Creates an LPOP command.
#[inline]
pub fn lpop(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("LPOP").arg(key)
}

/// Creates an RPOP command.
#[inline]
pub fn rpop(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("RPOP").arg(key)
}

/// Creates an LLEN command.
#[inline]
pub fn llen(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("LLEN").arg(key)
}

/// Creates an LRANGE command.
#[inline]
pub fn lrange(key: impl Into<Bytes>, start: i64, stop: i64) -> Cmd {
    Cmd::new("LRANGE")
        .arg(key)
        .arg(start.to_string())
        .arg(stop.to_string())
}

/// Creates an LINDEX command.
#[inline]
pub fn lindex(key: impl Into<Bytes>, index: i64) -> Cmd {
    Cmd::new("LINDEX").arg(key).arg(index.to_string())
}

/// Creates an LSET command.
#[inline]
pub fn lset(key: impl Into<Bytes>, index: i64, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("LSET").arg(key).arg(index.to_string()).arg(value)
}

/// Creates an LREM command.
#[inline]
pub fn lrem(key: impl Into<Bytes>, count: i64, value: impl Into<Bytes>) -> Cmd {
    Cmd::new("LREM").arg(key).arg(count.to_string()).arg(value)
}

/// Creates an LTRIM command.
#[inline]
pub fn ltrim(key: impl Into<Bytes>, start: i64, stop: i64) -> Cmd {
    Cmd::new("LTRIM")
        .arg(key)
        .arg(start.to_string())
        .arg(stop.to_string())
}

/// Creates an RPOPLPUSH command.
#[inline]
pub fn rpoplpush(source: impl Into<Bytes>, destination: impl Into<Bytes>) -> Cmd {
    Cmd::new("RPOPLPUSH").arg(source).arg(destination)
}

/// Creates a BLPOP command.
#[inline]
pub fn blpop(keys: Vec<String>, timeout: u64) -> Cmd {
    let mut cmd = Cmd::new("BLPOP");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd = cmd.arg(timeout.to_string());
    cmd
}

/// Creates a BRPOP command.
#[inline]
pub fn brpop(keys: Vec<String>, timeout: u64) -> Cmd {
    let mut cmd = Cmd::new("BRPOP");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd = cmd.arg(timeout.to_string());
    cmd
}

/// Creates an LPOS command.
#[inline]
pub fn lpos(key: impl Into<Bytes>, element: impl Into<Bytes>) -> Cmd {
    Cmd::new("LPOS").arg(key).arg(element)
}

/// Creates a SADD command.
#[inline]
pub fn sadd(key: String, members: Vec<Bytes>) -> Cmd {
    let mut cmd = Cmd::new("SADD").arg(key);
    for member in members {
        cmd = cmd.arg(member);
    }
    cmd
}

/// Creates a SREM command.
#[inline]
pub fn srem(key: String, members: Vec<Bytes>) -> Cmd {
    let mut cmd = Cmd::new("SREM").arg(key);
    for member in members {
        cmd = cmd.arg(member);
    }
    cmd
}

/// Creates a SPOP command.
#[inline]
pub fn spop(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("SPOP").arg(key)
}

/// Creates a SMEMBERS command.
#[inline]
pub fn smembers(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("SMEMBERS").arg(key)
}

/// Creates a SISMEMBER command.
#[inline]
pub fn sismember(key: impl Into<Bytes>, member: impl Into<Bytes>) -> Cmd {
    Cmd::new("SISMEMBER").arg(key).arg(member)
}

/// Creates a SCARD command.
#[inline]
pub fn scard(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("SCARD").arg(key)
}

/// Creates a SRANDMEMBER command.
#[inline]
pub fn srandmember(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("SRANDMEMBER").arg(key)
}

/// Creates a SDIFF command.
#[inline]
pub fn sdiff(keys: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("SDIFF");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd
}

/// Creates a SINTER command.
#[inline]
pub fn sinter(keys: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("SINTER");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd
}

/// Creates a SUNION command.
#[inline]
pub fn sunion(keys: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("SUNION");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd
}

/// Creates a SDIFFSTORE command.
#[inline]
pub fn sdiffstore(destination: String, keys: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("SDIFFSTORE").arg(destination);
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd
}

/// Creates a SINTERSTORE command.
#[inline]
pub fn sinterstore(destination: String, keys: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("SINTERSTORE").arg(destination);
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd
}

/// Creates a SUNIONSTORE command.
#[inline]
pub fn sunionstore(destination: String, keys: Vec<String>) -> Cmd {
    let mut cmd = Cmd::new("SUNIONSTORE").arg(destination);
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd
}

/// Creates a ZADD command.
#[inline]
pub fn zadd(key: String, members: Vec<(f64, Bytes)>) -> Cmd {
    let mut cmd = Cmd::new("ZADD").arg(key);
    for (score, member) in members {
        cmd = cmd.arg(score.to_string()).arg(member);
    }
    cmd
}

/// Creates a ZREM command.
#[inline]
pub fn zrem(key: String, members: Vec<Bytes>) -> Cmd {
    let mut cmd = Cmd::new("ZREM").arg(key);
    for member in members {
        cmd = cmd.arg(member);
    }
    cmd
}

/// Creates a ZRANGE command.
#[inline]
pub fn zrange(key: impl Into<Bytes>, start: i64, stop: i64) -> Cmd {
    Cmd::new("ZRANGE")
        .arg(key)
        .arg(start.to_string())
        .arg(stop.to_string())
}

/// Creates a ZRANGEBYSCORE command.
#[inline]
pub fn zrangebyscore(key: impl Into<Bytes>, min: impl Into<Bytes>, max: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZRANGEBYSCORE").arg(key).arg(min).arg(max)
}

/// Creates a ZRANK command.
#[inline]
pub fn zrank(key: impl Into<Bytes>, member: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZRANK").arg(key).arg(member)
}

/// Creates a ZSCORE command.
#[inline]
pub fn zscore(key: impl Into<Bytes>, member: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZSCORE").arg(key).arg(member)
}

/// Creates a ZCARD command.
#[inline]
pub fn zcard(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZCARD").arg(key)
}

/// Creates a ZCOUNT command.
#[inline]
pub fn zcount(key: impl Into<Bytes>, min: impl Into<Bytes>, max: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZCOUNT").arg(key).arg(min).arg(max)
}

/// Creates a ZINCRBY command.
#[inline]
pub fn zincrby(key: impl Into<Bytes>, increment: f64, member: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZINCRBY")
        .arg(key)
        .arg(increment.to_string())
        .arg(member)
}

/// Creates a ZREVRANGE command.
#[inline]
pub fn zrevrange(key: impl Into<Bytes>, start: i64, stop: i64) -> Cmd {
    Cmd::new("ZREVRANGE")
        .arg(key)
        .arg(start.to_string())
        .arg(stop.to_string())
}

/// Creates a ZREVRANK command.
#[inline]
pub fn zrevrank(key: impl Into<Bytes>, member: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZREVRANK").arg(key).arg(member)
}

/// Creates a ZREMRANGEBYRANK command.
#[inline]
pub fn zremrangebyrank(key: impl Into<Bytes>, start: i64, stop: i64) -> Cmd {
    Cmd::new("ZREMRANGEBYRANK")
        .arg(key)
        .arg(start.to_string())
        .arg(stop.to_string())
}

/// Creates a ZREMRANGEBYSCORE command.
#[inline]
pub fn zremrangebyscore(
    key: impl Into<Bytes>,
    min: impl Into<Bytes>,
    max: impl Into<Bytes>,
) -> Cmd {
    Cmd::new("ZREMRANGEBYSCORE").arg(key).arg(min).arg(max)
}

/// Creates a ZPOPMIN command.
#[inline]
pub fn zpopmin(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZPOPMIN").arg(key)
}

/// Creates a ZPOPMAX command.
#[inline]
pub fn zpopmax(key: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZPOPMAX").arg(key)
}

/// Creates a BZPOPMIN command.
#[inline]
pub fn bzpopmin(keys: Vec<String>, timeout: u64) -> Cmd {
    let mut cmd = Cmd::new("BZPOPMIN");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd = cmd.arg(timeout.to_string());
    cmd
}

/// Creates a BZPOPMAX command.
#[inline]
pub fn bzpopmax(keys: Vec<String>, timeout: u64) -> Cmd {
    let mut cmd = Cmd::new("BZPOPMAX");
    for key in keys {
        cmd = cmd.arg(key);
    }
    cmd = cmd.arg(timeout.to_string());
    cmd
}

/// Creates a ZLEXCOUNT command.
#[inline]
pub fn zlexcount(key: impl Into<Bytes>, min: impl Into<Bytes>, max: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZLEXCOUNT").arg(key).arg(min).arg(max)
}

/// Creates a ZRANGEBYLEX command.
#[inline]
pub fn zrangebylex(key: impl Into<Bytes>, min: impl Into<Bytes>, max: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZRANGEBYLEX").arg(key).arg(min).arg(max)
}

/// Creates a ZREMRANGEBYLEX command.
#[inline]
pub fn zremrangebylex(key: impl Into<Bytes>, min: impl Into<Bytes>, max: impl Into<Bytes>) -> Cmd {
    Cmd::new("ZREMRANGEBYLEX").arg(key).arg(min).arg(max)
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

/// Converts a frame array to a vector of optional bytes.
#[inline]
pub fn frame_to_vec_bytes(frame: Frame) -> Result<Vec<Option<Bytes>>, crate::Error> {
    match frame {
        Frame::Array(arr) => {
            let mut result = Vec::with_capacity(arr.len());
            for item in arr {
                match item {
                    Frame::BulkString(b) => result.push(b),
                    Frame::Null => result.push(None),
                    Frame::Error(e) => {
                        return Err(crate::Error::Server {
                            message: String::from_utf8_lossy(&e).into_owned(),
                        })
                    }
                    _ => {
                        return Err(crate::Error::Protocol {
                            message: "unexpected frame type in array".to_string(),
                        })
                    }
                }
            }
            Ok(result)
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "expected array frame".to_string(),
        }),
    }
}

/// Converts a frame to a string.
#[inline]
pub fn frame_to_string(frame: Frame) -> Result<String, crate::Error> {
    match frame {
        Frame::SimpleString(s) | Frame::Error(s) => Ok(String::from_utf8_lossy(&s).into_owned()),
        Frame::BulkString(Some(b)) => Ok(String::from_utf8_lossy(&b).into_owned()),
        Frame::BulkString(None) | Frame::Null => Ok(String::new()),
        Frame::Integer(i) => Ok(i.to_string()),
        _ => Err(crate::Error::Protocol {
            message: "unexpected frame type".to_string(),
        }),
    }
}

/// Converts a frame array to a SCAN response (cursor, keys).
#[inline]
pub fn frame_to_scan_response(frame: Frame) -> Result<(u64, Vec<String>), crate::Error> {
    match frame {
        Frame::Array(mut arr) => {
            if arr.len() != 2 {
                return Err(crate::Error::Protocol {
                    message: "SCAN response must have 2 elements".to_string(),
                });
            }

            let keys_frame = arr.pop().unwrap();
            let cursor_frame = arr.pop().unwrap();

            let cursor_str = frame_to_string(cursor_frame)?;
            let cursor = cursor_str
                .parse::<u64>()
                .map_err(|_| crate::Error::Protocol {
                    message: "invalid cursor value".to_string(),
                })?;

            let keys = match keys_frame {
                Frame::Array(key_arr) => {
                    let mut keys = Vec::with_capacity(key_arr.len());
                    for key_frame in key_arr {
                        keys.push(frame_to_string(key_frame)?);
                    }
                    keys
                }
                _ => {
                    return Err(crate::Error::Protocol {
                        message: "SCAN keys must be an array".to_string(),
                    })
                }
            };

            Ok((cursor, keys))
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "expected array frame for SCAN".to_string(),
        }),
    }
}

/// Converts a frame array to a vector of strings.
#[inline]
pub fn frame_to_vec_string(frame: Frame) -> Result<Vec<String>, crate::Error> {
    match frame {
        Frame::Array(arr) => {
            let mut result = Vec::with_capacity(arr.len());
            for item in arr {
                result.push(frame_to_string(item)?);
            }
            Ok(result)
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "expected array frame".to_string(),
        }),
    }
}

/// Converts a frame array to a hashmap (HGETALL response).
#[inline]
pub fn frame_to_hashmap(
    frame: Frame,
) -> Result<std::collections::HashMap<String, Bytes>, crate::Error> {
    match frame {
        Frame::Array(arr) => {
            if arr.len() % 2 != 0 {
                return Err(crate::Error::Protocol {
                    message: "HGETALL response must have even number of elements".to_string(),
                });
            }

            let mut result = std::collections::HashMap::new();
            let mut iter = arr.into_iter();

            while let Some(key_frame) = iter.next() {
                let value_frame = iter.next().unwrap();
                let key = frame_to_string(key_frame)?;
                let value = match value_frame {
                    Frame::BulkString(Some(b)) => b,
                    Frame::BulkString(None) | Frame::Null => Bytes::new(),
                    Frame::Error(e) => {
                        return Err(crate::Error::Server {
                            message: String::from_utf8_lossy(&e).into_owned(),
                        })
                    }
                    _ => {
                        return Err(crate::Error::Protocol {
                            message: "unexpected value frame type".to_string(),
                        })
                    }
                };
                result.insert(key, value);
            }

            Ok(result)
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "expected array frame for HGETALL".to_string(),
        }),
    }
}

/// Converts a frame to a float.
#[inline]
pub fn frame_to_float(frame: Frame) -> Result<f64, crate::Error> {
    match frame {
        Frame::BulkString(Some(b)) => {
            let s = String::from_utf8_lossy(&b);
            s.parse::<f64>().map_err(|_| crate::Error::Protocol {
                message: "invalid float value".to_string(),
            })
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "expected bulk string for float".to_string(),
        }),
    }
}

/// Converts a frame array to a vector of bytes (for LRANGE).
#[inline]
pub fn frame_to_vec_bytes_list(frame: Frame) -> Result<Vec<Bytes>, crate::Error> {
    match frame {
        Frame::Array(arr) => {
            let mut result = Vec::with_capacity(arr.len());
            for item in arr {
                match item {
                    Frame::BulkString(Some(b)) => result.push(b),
                    Frame::Error(e) => {
                        return Err(crate::Error::Server {
                            message: String::from_utf8_lossy(&e).into_owned(),
                        })
                    }
                    _ => {
                        return Err(crate::Error::Protocol {
                            message: "unexpected frame type in list array".to_string(),
                        })
                    }
                }
            }
            Ok(result)
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "expected array frame for list".to_string(),
        }),
    }
}

/// Converts a frame to a BLPOP/BRPOP response (key, value).
#[inline]
pub fn frame_to_blocking_pop(frame: Frame) -> Result<Option<(String, Bytes)>, crate::Error> {
    match frame {
        Frame::Null => Ok(None),
        Frame::Array(mut arr) => {
            if arr.len() != 2 {
                return Err(crate::Error::Protocol {
                    message: "BLPOP/BRPOP response must have 2 elements".to_string(),
                });
            }

            let value_frame = arr.pop().unwrap();
            let key_frame = arr.pop().unwrap();

            let key = frame_to_string(key_frame)?;
            let value = match value_frame {
                Frame::BulkString(Some(b)) => b,
                _ => {
                    return Err(crate::Error::Protocol {
                        message: "unexpected value frame type".to_string(),
                    })
                }
            };

            Ok(Some((key, value)))
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "unexpected frame type for blocking pop".to_string(),
        }),
    }
}

/// Converts a frame to an optional i64 (for ZRANK/ZREVRANK).
#[inline]
pub fn frame_to_optional_int(frame: Frame) -> Result<Option<i64>, crate::Error> {
    match frame {
        Frame::Null => Ok(None),
        Frame::Integer(i) => Ok(Some(i)),
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "unexpected frame type for optional int".to_string(),
        }),
    }
}

/// Converts a frame to an optional float (for ZSCORE).
#[inline]
pub fn frame_to_optional_float(frame: Frame) -> Result<Option<f64>, crate::Error> {
    match frame {
        Frame::Null => Ok(None),
        Frame::BulkString(None) => Ok(None),
        _ => frame_to_float(frame).map(Some),
    }
}

/// Converts a frame to a sorted set member with score (for ZPOPMIN/ZPOPMAX).
#[inline]
pub fn frame_to_zpop_result(frame: Frame) -> Result<Option<(String, f64)>, crate::Error> {
    match frame {
        Frame::Null => Ok(None),
        Frame::Array(mut arr) => {
            if arr.is_empty() {
                return Ok(None);
            }
            if arr.len() != 2 {
                return Err(crate::Error::Protocol {
                    message: "ZPOP response must have 2 elements".to_string(),
                });
            }

            let score_frame = arr.pop().unwrap();
            let member_frame = arr.pop().unwrap();

            let member = frame_to_string(member_frame)?;
            let score = frame_to_float(score_frame)?;

            Ok(Some((member, score)))
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "unexpected frame type for ZPOP".to_string(),
        }),
    }
}

/// Converts a frame to a BZPOPMIN/BZPOPMAX response (key, member, score).
#[inline]
pub fn frame_to_bzpop_result(frame: Frame) -> Result<Option<(String, String, f64)>, crate::Error> {
    match frame {
        Frame::Null => Ok(None),
        Frame::Array(mut arr) => {
            if arr.len() != 3 {
                return Err(crate::Error::Protocol {
                    message: "BZPOP response must have 3 elements".to_string(),
                });
            }

            let score_frame = arr.pop().unwrap();
            let member_frame = arr.pop().unwrap();
            let key_frame = arr.pop().unwrap();

            let key = frame_to_string(key_frame)?;
            let member = frame_to_string(member_frame)?;
            let score = frame_to_float(score_frame)?;

            Ok(Some((key, member, score)))
        }
        Frame::Error(e) => Err(crate::Error::Server {
            message: String::from_utf8_lossy(&e).into_owned(),
        }),
        _ => Err(crate::Error::Protocol {
            message: "unexpected frame type for BZPOP".to_string(),
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

    #[test]
    fn test_mget_cmd() {
        let cmd = mget(vec!["key1".to_string(), "key2".to_string()]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("MGET".into())),
                Frame::BulkString(Some("key1".into())),
                Frame::BulkString(Some("key2".into()))
            ])
        );
    }

    #[test]
    fn test_mset_cmd() {
        let cmd = mset(vec![
            ("key1".to_string(), Bytes::from("value1")),
            ("key2".to_string(), Bytes::from("value2")),
        ]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("MSET".into())),
                Frame::BulkString(Some("key1".into())),
                Frame::BulkString(Some("value1".into())),
                Frame::BulkString(Some("key2".into())),
                Frame::BulkString(Some("value2".into()))
            ])
        );
    }

    #[test]
    fn test_setnx_cmd() {
        let cmd = setnx("key", "value");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SETNX".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("value".into()))
            ])
        );
    }

    #[test]
    fn test_setex_cmd() {
        let cmd = setex("key", 60, "value");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SETEX".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("60".into())),
                Frame::BulkString(Some("value".into()))
            ])
        );
    }

    #[test]
    fn test_getdel_cmd() {
        let cmd = getdel("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("GETDEL".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_append_cmd() {
        let cmd = append("key", "value");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("APPEND".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("value".into()))
            ])
        );
    }

    #[test]
    fn test_strlen_cmd() {
        let cmd = strlen("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("STRLEN".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_frame_to_vec_bytes() {
        let frame = Frame::Array(vec![
            Frame::BulkString(Some("value1".into())),
            Frame::Null,
            Frame::BulkString(Some("value3".into())),
        ]);
        let result = frame_to_vec_bytes(frame).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Some(Bytes::from("value1")));
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(Bytes::from("value3")));
    }

    #[test]
    fn test_exists_cmd() {
        let cmd = exists(vec!["key1".to_string(), "key2".to_string()]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("EXISTS".into())),
                Frame::BulkString(Some("key1".into())),
                Frame::BulkString(Some("key2".into()))
            ])
        );
    }

    #[test]
    fn test_key_type_cmd() {
        let cmd = key_type("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("TYPE".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_expire_cmd() {
        let cmd = expire("key", 60);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("EXPIRE".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("60".into()))
            ])
        );
    }

    #[test]
    fn test_expireat_cmd() {
        let cmd = expireat("key", 1735689600);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("EXPIREAT".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("1735689600".into()))
            ])
        );
    }

    #[test]
    fn test_ttl_cmd() {
        let cmd = ttl("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("TTL".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_persist_cmd() {
        let cmd = persist("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("PERSIST".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_rename_cmd() {
        let cmd = rename("oldkey", "newkey");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("RENAME".into())),
                Frame::BulkString(Some("oldkey".into())),
                Frame::BulkString(Some("newkey".into()))
            ])
        );
    }

    #[test]
    fn test_scan_cmd() {
        let cmd = scan(0);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SCAN".into())),
                Frame::BulkString(Some("0".into()))
            ])
        );
    }

    #[test]
    fn test_frame_to_scan_response() {
        let frame = Frame::Array(vec![
            Frame::BulkString(Some("10".into())),
            Frame::Array(vec![
                Frame::BulkString(Some("key1".into())),
                Frame::BulkString(Some("key2".into())),
            ]),
        ]);
        let (cursor, keys) = frame_to_scan_response(frame).unwrap();
        assert_eq!(cursor, 10);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], "key1");
        assert_eq!(keys[1], "key2");
    }

    #[test]
    fn test_hset_cmd() {
        let cmd = hset("key", "field", "value");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HSET".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field".into())),
                Frame::BulkString(Some("value".into()))
            ])
        );
    }

    #[test]
    fn test_hget_cmd() {
        let cmd = hget("key", "field");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HGET".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field".into()))
            ])
        );
    }

    #[test]
    fn test_hmset_cmd() {
        let cmd = hmset(
            "key".to_string(),
            vec![
                ("field1".to_string(), Bytes::from("value1")),
                ("field2".to_string(), Bytes::from("value2")),
            ],
        );
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HMSET".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field1".into())),
                Frame::BulkString(Some("value1".into())),
                Frame::BulkString(Some("field2".into())),
                Frame::BulkString(Some("value2".into()))
            ])
        );
    }

    #[test]
    fn test_hmget_cmd() {
        let cmd = hmget(
            "key".to_string(),
            vec!["field1".to_string(), "field2".to_string()],
        );
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HMGET".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field1".into())),
                Frame::BulkString(Some("field2".into()))
            ])
        );
    }

    #[test]
    fn test_hgetall_cmd() {
        let cmd = hgetall("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HGETALL".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_hdel_cmd() {
        let cmd = hdel(
            "key".to_string(),
            vec!["field1".to_string(), "field2".to_string()],
        );
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HDEL".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field1".into())),
                Frame::BulkString(Some("field2".into()))
            ])
        );
    }

    #[test]
    fn test_hexists_cmd() {
        let cmd = hexists("key", "field");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HEXISTS".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field".into()))
            ])
        );
    }

    #[test]
    fn test_hlen_cmd() {
        let cmd = hlen("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HLEN".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_hkeys_cmd() {
        let cmd = hkeys("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HKEYS".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_hvals_cmd() {
        let cmd = hvals("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HVALS".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_hincrby_cmd() {
        let cmd = hincrby("key", "field", 5);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HINCRBY".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field".into())),
                Frame::BulkString(Some("5".into()))
            ])
        );
    }

    #[test]
    fn test_hincrbyfloat_cmd() {
        let cmd = hincrbyfloat("key", "field", 2.5);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HINCRBYFLOAT".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field".into())),
                Frame::BulkString(Some("2.5".into()))
            ])
        );
    }

    #[test]
    fn test_hsetnx_cmd() {
        let cmd = hsetnx("key", "field", "value");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("HSETNX".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("field".into())),
                Frame::BulkString(Some("value".into()))
            ])
        );
    }

    #[test]
    fn test_frame_to_hashmap() {
        let frame = Frame::Array(vec![
            Frame::BulkString(Some("field1".into())),
            Frame::BulkString(Some("value1".into())),
            Frame::BulkString(Some("field2".into())),
            Frame::BulkString(Some("value2".into())),
        ]);
        let result = frame_to_hashmap(frame).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("field1"), Some(&Bytes::from("value1")));
        assert_eq!(result.get("field2"), Some(&Bytes::from("value2")));
    }

    #[test]
    fn test_frame_to_vec_string() {
        let frame = Frame::Array(vec![
            Frame::BulkString(Some("str1".into())),
            Frame::BulkString(Some("str2".into())),
        ]);
        let result = frame_to_vec_string(frame).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "str1");
        assert_eq!(result[1], "str2");
    }

    #[test]
    fn test_lpush_cmd() {
        let cmd = lpush(
            "key".to_string(),
            vec![Bytes::from("val1"), Bytes::from("val2")],
        );
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("LPUSH".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("val1".into())),
                Frame::BulkString(Some("val2".into()))
            ])
        );
    }

    #[test]
    fn test_rpush_cmd() {
        let cmd = rpush("key".to_string(), vec![Bytes::from("val1")]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("RPUSH".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("val1".into()))
            ])
        );
    }

    #[test]
    fn test_lrange_cmd() {
        let cmd = lrange("key", 0, -1);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("LRANGE".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("0".into())),
                Frame::BulkString(Some("-1".into()))
            ])
        );
    }

    #[test]
    fn test_lrem_cmd() {
        let cmd = lrem("key", 2, "value");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("LREM".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("2".into())),
                Frame::BulkString(Some("value".into()))
            ])
        );
    }

    #[test]
    fn test_blpop_cmd() {
        let cmd = blpop(vec!["key1".to_string(), "key2".to_string()], 5);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("BLPOP".into())),
                Frame::BulkString(Some("key1".into())),
                Frame::BulkString(Some("key2".into())),
                Frame::BulkString(Some("5".into()))
            ])
        );
    }

    #[test]
    fn test_frame_to_blocking_pop() {
        let frame = Frame::Array(vec![
            Frame::BulkString(Some("mylist".into())),
            Frame::BulkString(Some("value".into())),
        ]);
        let result = frame_to_blocking_pop(frame).unwrap();
        assert!(result.is_some());
        let (key, value) = result.unwrap();
        assert_eq!(key, "mylist");
        assert_eq!(value, Bytes::from("value"));
    }

    #[test]
    fn test_sadd_cmd() {
        let cmd = sadd("key".to_string(), vec![Bytes::from("a"), Bytes::from("b")]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SADD".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("a".into())),
                Frame::BulkString(Some("b".into()))
            ])
        );
    }

    #[test]
    fn test_srem_cmd() {
        let cmd = srem("key".to_string(), vec![Bytes::from("a")]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SREM".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("a".into()))
            ])
        );
    }

    #[test]
    fn test_smembers_cmd() {
        let cmd = smembers("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SMEMBERS".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_sismember_cmd() {
        let cmd = sismember("key", "member");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SISMEMBER".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("member".into()))
            ])
        );
    }

    #[test]
    fn test_sdiff_cmd() {
        let cmd = sdiff(vec!["key1".to_string(), "key2".to_string()]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SDIFF".into())),
                Frame::BulkString(Some("key1".into())),
                Frame::BulkString(Some("key2".into()))
            ])
        );
    }

    #[test]
    fn test_sinter_cmd() {
        let cmd = sinter(vec!["key1".to_string(), "key2".to_string()]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SINTER".into())),
                Frame::BulkString(Some("key1".into())),
                Frame::BulkString(Some("key2".into()))
            ])
        );
    }

    #[test]
    fn test_sdiffstore_cmd() {
        let cmd = sdiffstore("dest".to_string(), vec!["key1".to_string()]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("SDIFFSTORE".into())),
                Frame::BulkString(Some("dest".into())),
                Frame::BulkString(Some("key1".into()))
            ])
        );
    }

    #[test]
    fn test_zadd_cmd() {
        let cmd = zadd(
            "key".to_string(),
            vec![(1.0, Bytes::from("a")), (2.5, Bytes::from("b"))],
        );
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZADD".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("1".into())),
                Frame::BulkString(Some("a".into())),
                Frame::BulkString(Some("2.5".into())),
                Frame::BulkString(Some("b".into()))
            ])
        );
    }

    #[test]
    fn test_zrem_cmd() {
        let cmd = zrem("key".to_string(), vec![Bytes::from("a"), Bytes::from("b")]);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZREM".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("a".into())),
                Frame::BulkString(Some("b".into()))
            ])
        );
    }

    #[test]
    fn test_zrange_cmd() {
        let cmd = zrange("key", 0, 10);
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZRANGE".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("0".into())),
                Frame::BulkString(Some("10".into()))
            ])
        );
    }

    #[test]
    fn test_zrangebyscore_cmd() {
        let cmd = zrangebyscore("key", "-inf", "+inf");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZRANGEBYSCORE".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("-inf".into())),
                Frame::BulkString(Some("+inf".into()))
            ])
        );
    }

    #[test]
    fn test_zrank_cmd() {
        let cmd = zrank("key", "member");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZRANK".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("member".into()))
            ])
        );
    }

    #[test]
    fn test_zscore_cmd() {
        let cmd = zscore("key", "member");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZSCORE".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("member".into()))
            ])
        );
    }

    #[test]
    fn test_zcard_cmd() {
        let cmd = zcard("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZCARD".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_zcount_cmd() {
        let cmd = zcount("key", "0", "100");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZCOUNT".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("0".into())),
                Frame::BulkString(Some("100".into()))
            ])
        );
    }

    #[test]
    fn test_zincrby_cmd() {
        let cmd = zincrby("key", 2.5, "member");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZINCRBY".into())),
                Frame::BulkString(Some("key".into())),
                Frame::BulkString(Some("2.5".into())),
                Frame::BulkString(Some("member".into()))
            ])
        );
    }

    #[test]
    fn test_zpopmin_cmd() {
        let cmd = zpopmin("key");
        assert_eq!(
            cmd.into_frame(),
            Frame::Array(vec![
                Frame::BulkString(Some("ZPOPMIN".into())),
                Frame::BulkString(Some("key".into()))
            ])
        );
    }

    #[test]
    fn test_frame_to_optional_int() {
        let frame = Frame::Integer(42);
        let result = frame_to_optional_int(frame).unwrap();
        assert_eq!(result, Some(42));

        let null_frame = Frame::Null;
        let null_result = frame_to_optional_int(null_frame).unwrap();
        assert_eq!(null_result, None);
    }

    #[test]
    fn test_frame_to_optional_float() {
        let frame = Frame::BulkString(Some(Bytes::from("2.5")));
        let result = frame_to_optional_float(frame).unwrap();
        assert_eq!(result, Some(2.5));

        let null_frame = Frame::Null;
        let null_result = frame_to_optional_float(null_frame).unwrap();
        assert_eq!(null_result, None);
    }

    #[test]
    fn test_frame_to_zpop_result() {
        let frame = Frame::Array(vec![
            Frame::BulkString(Some("member".into())),
            Frame::BulkString(Some("1.5".into())),
        ]);
        let result = frame_to_zpop_result(frame).unwrap();
        assert!(result.is_some());
        let (member, score) = result.unwrap();
        assert_eq!(member, "member");
        assert_eq!(score, 1.5);

        let null_frame = Frame::Null;
        let null_result = frame_to_zpop_result(null_frame).unwrap();
        assert_eq!(null_result, None);
    }

    #[test]
    fn test_frame_to_bzpop_result() {
        let frame = Frame::Array(vec![
            Frame::BulkString(Some("key".into())),
            Frame::BulkString(Some("member".into())),
            Frame::BulkString(Some("2.0".into())),
        ]);
        let result = frame_to_bzpop_result(frame).unwrap();
        assert!(result.is_some());
        let (key, member, score) = result.unwrap();
        assert_eq!(key, "key");
        assert_eq!(member, "member");
        assert_eq!(score, 2.0);

        let null_frame = Frame::Null;
        let null_result = frame_to_bzpop_result(null_frame).unwrap();
        assert_eq!(null_result, None);
    }
}
