use bytes::Bytes;
use crate::network::{NetworkResult, NetworkUpdate};

pub trait Network {
    fn send(&mut self, source: i32, msg: Bytes) -> NetworkResult<()>;
    fn receive(&mut self) -> NetworkResult<NetworkUpdate>;
}
