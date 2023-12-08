use crate::message::Message;
use crate::state::States;
use std::collections::HashMap;

pub mod factory;

/// This trait represents the mailbox of a device. It is used to store the messages received from the neighbors
pub trait Mailbox {
    /// Enqueue a message in the mailbox
    fn enqueue(&mut self, msg: Message);
    /// Returns the messages stored in the mailbox
    fn messages(&mut self) -> Messages;
}

/// This type alias represent the messages stored in the mailbox
pub type Messages = HashMap<i32, Message>;

/// This trait is used to convert a set of [Messages] into a set of [States]
pub trait AsStates {
    fn as_states(&self) -> States;
}

impl AsStates for Messages {
    fn as_states(&self) -> States {
        let mut states = States::new();
        for (id, msg) in self.iter() {
            states.insert(*id, msg.export.clone());
        }
        states
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::time::SystemTime;

    #[test]
    fn test_as_states() {
        use crate::mailbox::AsStates;
        use crate::message::Message;
        use crate::state::States;
        use rf_core::export;
        use rf_core::export::Export;
        use rf_core::path::Path;
        use std::any::Any;

        let mut messages = HashMap::new();
        let export_1 = export!((Path::new(), 1));
        let export_2 = export!((Path::new(), 2));
        let export_3 = export!((Path::new(), 3));
        messages.insert(1, Message::new(1, export_1.clone(), SystemTime::now()));
        messages.insert(2, Message::new(2, export_2.clone(), SystemTime::now()));
        messages.insert(3, Message::new(3, export_3.clone(), SystemTime::now()));

        let states: States = messages.as_states();
        assert_eq!(states.len(), 3);
        assert_eq!(states.get(&1).unwrap(), &export_1);
        assert_eq!(states.get(&2).unwrap(), &export_2);
        assert_eq!(states.get(&3).unwrap(), &export_3);
    }
}
