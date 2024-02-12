use std::time::Duration;
use async_trait::async_trait;
use rf_distributed::time::Time;

pub struct TimeImpl;

impl TimeImpl {
    pub fn new() -> Self {
        TimeImpl
    }
}

#[async_trait]
impl Time for TimeImpl {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}