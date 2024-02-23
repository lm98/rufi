use crate::network::NetworkResult;
use async_trait::async_trait;
use bytes::Bytes;
use crate::mailbox::Messages;

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
    async fn send(&mut self, source: i32, msg: Bytes) -> NetworkResult<()>;
    /// Receive the messages from the network
    ///
    /// # Returns
    ///
    /// * `Messages` - the messages received
    async fn receive(&mut self) -> Messages;
}
