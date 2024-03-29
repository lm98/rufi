use rf_distributed::mailbox::{Mailbox, Messages};
use rf_distributed::message::Message;
use std::collections::{BTreeMap, HashMap};
use std::time::SystemTime;

/// This struct is used as a factory for [Mailbox]es.
pub struct MailboxFactory;

impl MailboxFactory {
    /// Creates a new [MemoryLessMailbox] with a memory less message processing policy.
    ///For each neighbouring message, only the last one received is kept. This policy, from the user's viewpoint, acts similarly to
    /// the [MostRecent] version, but it is more memory efficient since the other messages received are substituted
    /// with the last received.
    pub fn memory_less() -> MemoryLessMailbox {
        MemoryLessMailbox::new()
    }

    /// Creates a new [TimeOrderedMailbox] with a most recent message processing policy.
    /// Keeps every message received from each neighbor, but returns only the most recent one. Used
    /// for processing in a LIFO order.
    pub fn most_recent() -> TimeOrderedMailbox {
        TimeOrderedMailbox {
            messages: HashMap::new(),
            pop_first: false,
        }
    }

    /// Creates a new [TimeOrderedMailbox] with a least recent message processing policy.
    /// Keeps every message received from each neighbor, but returns only the least recent one. Used
    /// for processing in a FIFO order.
    pub fn least_recent() -> TimeOrderedMailbox {
        TimeOrderedMailbox {
            messages: HashMap::new(),
            pop_first: true,
        }
    }
}

pub struct MemoryLessMailbox {
    messages: HashMap<i32, Message>,
}

impl MemoryLessMailbox {
    pub fn new() -> Self {
        MemoryLessMailbox {
            messages: HashMap::new(),
        }
    }
}

impl Default for MemoryLessMailbox {
    fn default() -> Self {
        MemoryLessMailbox::new()
    }
}

impl Mailbox for MemoryLessMailbox {
    fn enqueue(&mut self, msg: Message) {
        self.messages.insert(msg.source, msg);
    }

    fn messages(&mut self) -> Messages {
        self.messages.clone()
    }
}

pub struct TimeOrderedMailbox {
    messages: HashMap<i32, BTreeMap<SystemTime, Message>>,
    pop_first: bool,
}

impl Mailbox for TimeOrderedMailbox {
    fn enqueue(&mut self, msg: Message) {
        let msgs = self.messages.entry(msg.source).or_default();
        msgs.insert(msg.timestamp, msg);
    }

    fn messages(&mut self) -> Messages {
        let mut messages = HashMap::new();
        for (id, msgs) in self.messages.iter_mut() {
            if self.pop_first {
                //get the first entry of the BTreeMap
                if let Some((_, msg)) = msgs.pop_first() {
                    messages.insert(*id, msg.clone());
                }
            } else {
                //get the last entry of the BTreeMap
                if let Some((_, msg)) = msgs.pop_last() {
                    messages.insert(*id, msg.clone());
                }
            }
        }
        messages
    }
}

#[cfg(test)]
mod test {
    use crate::mailbox::MailboxFactory;
    use rf_core::export;
    use rf_core::export::Export;
    use rf_core::path::Path;
    use rf_distributed::mailbox::Mailbox;
    use rf_distributed::message::Message;
    use std::any::Any;
    use std::collections::HashMap;
    use std::time::SystemTime;

    #[test]
    fn test_memory_less() {
        let mut mailbox = MailboxFactory::memory_less();
        let export_2 = export!((Path::new(), 2));
        let export_3 = export!((Path::new(), 3));
        let msg_2 = Message::new(2, export_2.clone(), SystemTime::now());
        let msg_3 = Message::new(3, export_3.clone(), SystemTime::now());
        mailbox.enqueue(msg_2.clone());
        mailbox.enqueue(msg_3.clone());
        let messages = mailbox.messages();
        assert_eq!(messages, HashMap::from([(2, msg_2), (3, msg_3.clone())]));

        // update msg_2
        let new_export_2 = export!((Path::new(), 2 + 2));
        let new_msg_2 = Message::new(2, new_export_2, SystemTime::now());
        mailbox.enqueue(new_msg_2.clone());
        let messages = mailbox.messages();
        assert_eq!(messages, HashMap::from([(2, new_msg_2), (3, msg_3)]));
    }

    #[test]
    fn test_most_recent() {
        let mut mailbox = MailboxFactory::most_recent();

        // add the first round of messages
        let export_2 = export!((Path::new(), 2));
        let export_3 = export!((Path::new(), 3));
        let msg_2 = Message::new(2, export_2.clone(), SystemTime::now());
        let msg_3 = Message::new(3, export_3.clone(), SystemTime::now());
        mailbox.enqueue(msg_2.clone());
        mailbox.enqueue(msg_3.clone());

        // add the second round of messages
        let new_export_2 = export!((Path::new(), 2 + 2));
        let new_msg_2 = Message::new(2, new_export_2, SystemTime::now());
        let new_export_3 = export!((Path::new(), 3 + 3));
        let new_msg_3 = Message::new(3, new_export_3, SystemTime::now());
        mailbox.enqueue(new_msg_2.clone());
        mailbox.enqueue(new_msg_3.clone());

        // pop once
        let messages = mailbox.messages();
        assert_eq!(messages, HashMap::from([(2, new_msg_2), (3, new_msg_3)]));

        // pop the second time
        let messages = mailbox.messages();
        assert_eq!(messages, HashMap::from([(2, msg_2), (3, msg_3)]));
    }

    #[test]
    fn test_least_recent() {
        let mut mailbox = MailboxFactory::least_recent();

        // add the first round of messages
        let export_2 = export!((Path::new(), 2));
        let export_3 = export!((Path::new(), 3));
        let msg_2 = Message::new(2, export_2.clone(), SystemTime::now());
        let msg_3 = Message::new(3, export_3.clone(), SystemTime::now());
        mailbox.enqueue(msg_2.clone());
        mailbox.enqueue(msg_3.clone());

        // add the second round of messages
        let new_export_2 = export!((Path::new(), 2 + 2));
        let new_msg_2 = Message::new(2, new_export_2, SystemTime::now());
        let new_export_3 = export!((Path::new(), 3 + 3));
        let new_msg_3 = Message::new(3, new_export_3, SystemTime::now());
        mailbox.enqueue(new_msg_2.clone());
        mailbox.enqueue(new_msg_3.clone());

        // pop once
        let messages = mailbox.messages();
        assert_eq!(messages, HashMap::from([(2, msg_2), (3, msg_3)]));

        // pop the second time
        let messages = mailbox.messages();
        assert_eq!(messages, HashMap::from([(2, new_msg_2), (3, new_msg_3)]));
    }
}
