use bytes::Bytes;

pub mod asynchronous;
pub mod sync;

/// This type represent the result of a network operation
pub type NetworkResult<T> = Result<T, Box<dyn std::error::Error>>;

/// This enum represent the different update we will receive from the network
pub enum NetworkUpdate {
    /// No message received
    None,
    /// A message has been received
    Update { msg: Bytes },
    /// An error occurred
    Err { reason: String },
}
