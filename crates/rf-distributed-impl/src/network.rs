use bytes::Bytes;
use log::info;
use rf_distributed::network::{sync::Network, NetworkResult, NetworkUpdate};
use rumqttc::{Client, Event::Incoming, MqttOptions, QoS};
use std::error::Error;
use std::sync::mpsc::{channel, Receiver};
use std::thread;


/// This struct represent the network that will be used to send and receive messages
/// using the MQTT protocol.
pub struct SyncMQTTNetwork {
    client: Client,
    receiver: Receiver<NetworkUpdate>,
}

impl SyncMQTTNetwork {
    pub fn new(
        options: MqttOptions,
        topics: Vec<i32>,
        mqtt_channel_cap: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let (mut client, mut connection) = Client::new(options, mqtt_channel_cap);
        SyncMQTTNetwork::subscribe_to_topics(&mut client, topics)?;
        let (sender, receiver) = channel::<NetworkUpdate>();
        thread::spawn(move || {
            loop {
                for (_i, notification) in connection.iter().enumerate() {
                    match notification {
                        Ok(Incoming(rumqttc::Packet::Publish(msg))) => {
                            if let Err(send_error) = sender
                                .send(NetworkUpdate::Update { msg: msg.payload })
                            {
                                info!(
                                    "Error sending message to receiver: {:?}",
                                    send_error.to_string()
                                );
                            }
                        }
                        Ok(Incoming(rumqttc::Packet::Disconnect)) => {
                            sender
                                .send(NetworkUpdate::Err {
                                    reason: "Disconnected".to_string(),
                                }).unwrap_or(()); // Ignore the error
                        }
                        Err(e) => {
                            if let Err(e2) = sender
                                .send(NetworkUpdate::Err {
                                    reason: e.to_string(),
                                })
                            {
                                info!("Error: {:?}", e2.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
        Ok(Self { client, receiver })
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
    fn send(&mut self, source: i32, msg: Bytes) -> NetworkResult<()> {
        self.client
            .try_publish(
                format!("hello-rufi/{source}/subscriptions"),
                QoS::AtMostOnce,
                false,
                msg,
            )
            .map_err(|e| e.into())
    }

    fn receive(&mut self) -> NetworkResult<NetworkUpdate> {
        self.receiver
            .recv()
            .map_err(|_e| "No message received".into())
    }
}
