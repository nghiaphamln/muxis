use bytes::Bytes;
use std::time::Duration;

pub use muxis_proto::error::Error;

pub mod connection;
pub mod multiplexed;

#[derive(Debug)]
pub struct Client {
    // Placeholder for Phase 0
}

#[derive(Debug)]
pub struct ClientPool {
    // Placeholder for Phase 0
}

impl Client {
    pub async fn connect<T: AsRef<str>>(_addr: T) -> Result<Self, crate::Error> {
        Ok(Self {})
    }

    pub async fn ping(&mut self) -> Result<Bytes, crate::Error> {
        Ok("PONG".into())
    }

    pub async fn get(&mut self, _key: &str) -> Result<Option<Bytes>, crate::Error> {
        Ok(None)
    }

    pub async fn set(&mut self, _key: &str, _value: Bytes) -> Result<(), crate::Error> {
        Ok(())
    }

    pub async fn set_with_expiry(
        &mut self,
        _key: &str,
        _value: Bytes,
        _expiry: Duration,
    ) -> Result<(), crate::Error> {
        Ok(())
    }

    pub async fn incr(&mut self, _key: &str) -> Result<i64, crate::Error> {
        Ok(0)
    }

    pub async fn incr_by(&mut self, _key: &str, _amount: i64) -> Result<i64, crate::Error> {
        Ok(0)
    }

    pub async fn decr(&mut self, _key: &str) -> Result<i64, crate::Error> {
        Ok(0)
    }

    pub async fn decr_by(&mut self, _key: &str, _amount: i64) -> Result<i64, crate::Error> {
        Ok(0)
    }

    pub async fn del(&mut self, _key: &str) -> Result<bool, crate::Error> {
        Ok(false)
    }
}

impl ClientPool {
    pub fn new(_size: usize) -> Self {
        Self {}
    }
}
