use rf_distributed::time::Time;
use std::time::Duration;

pub struct TimeImpl;

impl TimeImpl {
    pub fn new() -> Self {
        TimeImpl
    }
}

impl Time for TimeImpl {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}
