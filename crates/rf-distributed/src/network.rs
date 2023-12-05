use async_trait::async_trait;

pub mod factory;

type NetworkResult<T> = Result<T, Box<dyn std::error::Error>>;

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
    async fn subscribe(&mut self, topics: Vec<i32>) -> NetworkResult<()>;

    async fn unsubscribe(&mut self, topics: Vec<i32>) -> NetworkResult<()>;
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
    async fn recv(&mut self) -> NetworkResult<NetworkUpdate>;
}