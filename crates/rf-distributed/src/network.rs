use async_trait::async_trait;

/// This type represent the result of a network operation
pub type NetworkResult<T> = Result<T, Box<dyn std::error::Error>>;

/// This enum represent the different update we will receive from the network
pub enum NetworkUpdate {
    /// No message received
    None,
    /// A message has been received
    Update { msg: String },
}

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
    async fn send(&self, source: i32, msg: String) -> NetworkResult<()>;
    /// Receive a message from the network
    ///
    /// # Returns
    ///
    /// * `Ok(NetworkUpdate)` - If a message has been received
    /// * `Err(e)` - If an error occurred
    async fn receive(&mut self) -> NetworkResult<NetworkUpdate>;
}
