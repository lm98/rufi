use crate::network::{Network, NetworkResult, NetworkUpdate};
use async_trait::async_trait;
use bytes::Bytes;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::sync::mpsc::Receiver;

/// This struct represent a factory that will be used to create a [Network]
pub struct NetworkFactory;

impl NetworkFactory {
    /// Create a new [Network] that will be used to send and receive messages, implemented via
    /// MQTT protocol.
    ///
    /// # Arguments
    ///
    /// * `options` - The options for the MQTT client
    /// * `topics` - The topics to subscribe to, in order to receive messages
    ///
    /// # Returns
    ///
    /// * `Box<dyn Network>` - The network created
    pub async fn async_mqtt_network(options: MqttOptions, topics: Vec<i32>) -> Box<dyn Network> {
        Box::new(AsyncMQTTNetwork::new(options, topics).await)
    }
}

struct AsyncMQTTNetwork {
    client: AsyncClient,
    receiver: Receiver<NetworkUpdate>,
}

impl AsyncMQTTNetwork {
    pub async fn new(options: MqttOptions, topics: Vec<i32>) -> Self {
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        AsyncMQTTNetwork::subscribe_to_topics(client.clone(), topics)
            .await
            .unwrap();
        let (sender, receiver) = tokio::sync::mpsc::channel::<NetworkUpdate>(100);
        tokio::spawn(async move {
            loop {
                if let Ok(notification) = eventloop.poll().await {
                    if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                        let msg_string = String::from_utf8(msg.payload.to_vec()).unwrap();
                        sender
                            .send(NetworkUpdate::Update { msg: msg_string })
                            .await
                            .unwrap();
                    }
                } else {
                    sender.send(NetworkUpdate::None).await.unwrap();
                }
            }
        });
        Self { client, receiver }
    }

    async fn subscribe_to_topics(client: AsyncClient, topics: Vec<i32>) -> NetworkResult<()> {
        for nbr in topics.clone() {
            client
                .subscribe(format!("hello-rufi/{nbr}/subscriptions"), QoS::AtMostOnce)
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Network for AsyncMQTTNetwork {
    async fn send(&self, source: i32, msg: String) -> NetworkResult<()> {
        self.client
            .publish(
                format!("hello-rufi/{source}/subscriptions"),
                QoS::AtMostOnce,
                false,
                Bytes::from(msg),
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
