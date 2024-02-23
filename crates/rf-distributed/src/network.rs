pub mod asynchronous;
pub mod sync;

/// This type represent the result of a network operation
pub type NetworkResult<T> = Result<T, Box<dyn std::error::Error>>;
