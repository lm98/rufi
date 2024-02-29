use crate::network::NetworkResult;
use crate::mailbox::Messages;
use crate::message::Message;

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
    fn send(&mut self, msg: Message) -> NetworkResult<()>;

    /// Receive the messages from the network
    ///
    /// # Returns
    ///
    /// * `Messages` - the messages received
    fn receive(&mut self) -> NetworkResult<Messages>;
}
