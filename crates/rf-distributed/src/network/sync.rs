use crate::network::NetworkResult;
use bytes::Bytes;
use crate::mailbox::Messages;

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
    fn send(&mut self, source: i32, msg: Bytes) -> NetworkResult<()>;

    /// Receive the messages from the network
    ///
    /// # Returns
    ///
    /// * `Messages` - the messages received
    fn receive(&mut self) -> Messages;
}
