use async_trait::async_trait;
use std::time::Duration;

/// This trait deals with time operations
#[async_trait]
pub trait Time {
    /// Sleep for the given duration
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration to sleep
    async fn sleep(&self, duration: Duration);
}
