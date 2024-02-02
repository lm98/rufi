use crate::network::{NetworkResult, NetworkUpdate};
use async_trait::async_trait;

/// This trait represent a network that will be used to send and receive messages
#[async_trait]
pub trait Network {
    /// Send a message to the network
    ///
    /// # Arguments
    ///
    /// * `source` - The source of the message
    /// * `msg` - The message to send
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the message has been sent
    /// * `Err(e)` - If an error occurred
    async fn send(&mut self, source: i32, msg: String) -> NetworkResult<()>;
    /// Receive a message from the network
    ///
    /// # Returns
    ///
    /// * `Ok(NetworkUpdate)` - If a message has been received
    /// * `Err(e)` - If an error occurred
    async fn receive(&mut self) -> NetworkResult<NetworkUpdate>;
}
