use std::error::Error;
use bytes::Bytes;
use rf_distributed::network::{sync::Network, NetworkResult};
use rumqttc::{Client, Event::Incoming, MqttOptions, QoS};
use std::sync::{Arc, Mutex};
use std::thread;
use rf_distributed::mailbox::{Mailbox, Messages};
use rf_distributed::message::Message;
use crate::mailbox::MemoryLessMailbox;


/// This struct represent the network that will be used to send and receive messages
/// using the MQTT protocol.
pub struct SyncMQTTNetwork {
    client: Client,
    mb: Arc<Mutex<Vec<Bytes>>>,
}

impl SyncMQTTNetwork {
    pub fn new(
        options: MqttOptions,
        topics: Vec<i32>,
        mqtt_channel_cap: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let (mut client, mut connection) = Client::new(options, mqtt_channel_cap);
        SyncMQTTNetwork::subscribe_to_topics(&mut client, topics)?;
        let mb: Arc<Mutex<Vec<Bytes>>> = Arc::new(Mutex::new(vec![]));

        let mb_clone = Arc::clone(&mb);
        thread::spawn(move || {
            loop {
                for (_i, notification) in connection.iter().enumerate() {
                    match notification {
                        Ok(Incoming(rumqttc::Packet::Publish(msg))) => {
                            if let Ok(mut mb) = mb_clone.lock() {
                                mb.push(msg.payload);
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
        Ok(Self { client, mb })
    }

    fn subscribe_to_topics(client: &mut Client, topics: Vec<i32>) -> NetworkResult<()> {
        for nbr in topics.clone() {
            if let Err(e) = client
                .subscribe(format!("hello-rufi/{nbr}/subscriptions"), QoS::AtMostOnce)
            {
                return Err(e.into());
            }
        }
        Ok(())
    }
}

impl Network for SyncMQTTNetwork {
    fn send(&mut self, msg: Message) -> NetworkResult<()> {
        let source = msg.source;
        let to_send = serde_json::to_vec(&msg)?;
        self.client
            .try_publish(
                format!("hello-rufi/{source}/subscriptions"),
                QoS::AtMostOnce,
                false,
                to_send,
            )
            .map_err(|e| e.into())
    }

    fn receive(&mut self) -> NetworkResult<Messages> {
        let mut mailbox = MemoryLessMailbox::new();

        for u in self.mb.lock().unwrap().iter() {
            if let Ok(mex) = serde_json::from_slice::<Message>(u) {
                mailbox.enqueue(mex)
            }
        }

        Ok(mailbox.messages())
    }
}
