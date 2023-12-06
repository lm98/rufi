use rufi::core::context::Context;
use rufi::core::sensor_id::{sensor, SensorId};
use rufi::programs::gradient;
use rumqttc::MqttOptions;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use rufi::distributed::discovery::Discovery;
use rufi::distributed::mailbox::factory::{MailboxFactory, ProcessingPolicy};
use rufi::distributed::network::factory::NetworkFactory;
use rufi::distributed::platform::RuFiPlatform;

#[derive(Debug, Default)]
struct Arguments {
    pub id: i32,
    pub source: bool,
}

impl Arguments {
    pub fn parse<S: AsRef<str>>(args: impl IntoIterator<Item = S>) -> Result<Arguments, String> {
        let mut r = Arguments::default();

        for arg in args {
            match arg.as_ref() {
                "-t" => r.source = true,
                "-f" => r.source = false,
                x => r.id = x.parse::<i32>().unwrap(),
            }
        }

        Ok(r)
    }
}

struct MockDiscovery(i32);

impl MockDiscovery {
    pub fn mock_discovery(id: i32) -> Box<dyn Discovery> {
        Box::new(MockDiscovery(id))
    }
}

impl Discovery for MockDiscovery {
    fn discover_neighbors(&self) -> Vec<i32> {
        let self_id = self.0;
        vec![self_id - 1, self_id, self_id + 1]
            .into_iter()
            .filter(|n| (n > &0 && n < &6))
            .collect()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get arguments from the CLI
    let args = Arguments::parse(std::env::args().skip(1))?;
    let self_id = args.id;
    let is_source = args.source;

    /* Set up a simple topology that will be used for these tests.
     *  Topology: [1] -- [2] -- [3] -- [4] -- [5].
     */
    let discovery = MockDiscovery::mock_discovery(self_id);
    let nbrs = discovery.discover_neighbors();

    // Setup the context
    let local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>> = vec![(
        sensor("source"),
        Rc::new(Box::new(is_source) as Box<dyn Any>),
    )]
    .into_iter()
    .collect();
    let nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>> = HashMap::from([(
        sensor("nbr_range"),
        nbrs.clone().iter()
            .map(|n| {
                (
                    n.clone(),
                    Rc::new(Box::new(i32::abs(self_id - n)) as Box<dyn Any>),
                )
            })
            .collect(),
    )]);
    let context = Context::new(
        self_id,
        local_sensor.clone(),
        nbr_sensor.clone(),
        Default::default(),
    );

    // Setup the MQTT client
    let mut mqttoptions =
        MqttOptions::new(format!("device#{}", self_id), "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let network = NetworkFactory::async_mqtt_network(mqttoptions, nbrs.clone()).await;
    // Setup the mailbox
    let mailbox = MailboxFactory::from_policy(ProcessingPolicy::MemoryLess);

    // Setup the platform and run the program
    RuFiPlatform::new(
        mailbox,
        network,
        context,
        discovery,
    ).run_forever(gradient).await
}
