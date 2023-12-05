use std::time::SystemTime;
use rf_core::export::Export;
use serde::{Deserialize, Serialize};

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
    use std::any::Any;
    use std::collections::HashMap;
    use rf_core::path::Path;
    use rf_core::export;
    use super::*;

    #[test]
    fn test_new() {
        let export = export!((Path::new(), 1));
        let msg = Message::new(1, export.clone(), SystemTime::now());
        assert_eq!(msg.source, 1);
        assert_eq!(msg.export, export);
    }
}
