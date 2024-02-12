use std::error::Error;
use async_trait::async_trait;
use bytes::Bytes;
use rf_distributed::network::{asynchronous::Network, NetworkResult, NetworkUpdate};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::sync::mpsc::Receiver;
use log::info;

/// This struct represent the network that will be used to send and receive messages
/// using the MQTT protocol.
pub struct AsyncMQTTNetwork {
    client: AsyncClient,
    receiver: Receiver<NetworkUpdate>,
}

impl AsyncMQTTNetwork {
    pub async fn new(options: MqttOptions, topics: Vec<i32>, mqtt_channel_cap: usize, network_buffer_size: usize) -> Result<Self, Box<dyn Error>> {
        let (client, mut eventloop) = AsyncClient::new(options, mqtt_channel_cap);
        AsyncMQTTNetwork::subscribe_to_topics(client.clone(), topics)
            .await?;
        let (sender, receiver) = tokio::sync::mpsc::channel::<NetworkUpdate>(network_buffer_size);
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(notification) => {
                        match notification {
                            rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) => {
                                if let Err(send_error) = sender
                                    .send(NetworkUpdate::Update { msg: msg.payload })
                                    .await {
                                    info!("Error sending message to receiver: {:?}", send_error.to_string());
                                }
                            }
                            _ => {

                            }
                        }
                    }
                    Err(e) => {
                        if let Err(e2) = sender.send(NetworkUpdate::Err { reason: e.to_string() }).await {
                            info!("Error: {:?}", e2.to_string());
                        }
                    }
                }
            }
        });
        Ok(Self { client, receiver })
    }

    async fn subscribe_to_topics(client: AsyncClient, topics: Vec<i32>) -> NetworkResult<()> {
        for nbr in topics.clone() {
            if let Err(e) = client
                .subscribe(format!("hello-rufi/{nbr}/subscriptions"), QoS::AtMostOnce)
                .await {
                return Err(e.into());
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Network for AsyncMQTTNetwork {
    async fn send(&mut self, source: i32, msg: Bytes) -> NetworkResult<()> {
        self.client
            .publish(
                format!("hello-rufi/{source}/subscriptions"),
                QoS::AtMostOnce,
                false,
                msg,
            )
            .await
            .map_err(|e| e.into())
    }

    async fn receive(&mut self) -> NetworkResult<NetworkUpdate> {
        self.receiver
            .recv()
            .await
            .ok_or("No message received".into())
    }
}
