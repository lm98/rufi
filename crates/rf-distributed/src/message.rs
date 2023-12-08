use rf_core::export::Export;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// This struct represent a message that will be sent between nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub source: i32,
    pub export: Export,
    pub timestamp: SystemTime,
}

impl Message {
    pub fn new(source: i32, p1: Export, sys_t: SystemTime) -> Self {
        Self {
            source,
            export: p1,
            timestamp: sys_t,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rf_core::export;
    use rf_core::path::Path;
    use std::any::Any;

    #[test]
    fn test_new() {
        let export = export!((Path::new(), 1));
        let msg = Message::new(1, export.clone(), SystemTime::now());
        assert_eq!(msg.source, 1);
        assert_eq!(msg.export, export);
    }
}
