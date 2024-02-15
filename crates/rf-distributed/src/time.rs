use std::time::Duration;

/// This trait deals with time operations
pub trait Time {
    /// Sleep for the given duration
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration to sleep
    fn sleep(&self, duration: Duration);
}
