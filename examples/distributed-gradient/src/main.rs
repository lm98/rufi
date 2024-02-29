use rufi::core::context::{Context, NbrSensors};
use rufi::core::sensor_id::{sensor, SensorId};
use rufi::distributed::discovery::nbr_sensors_setup::NbrSensorSetup;
use rufi::distributed::discovery::Discovery;
use rufi::distributed::impls::network::SyncMQTTNetwork;
use rufi::distributed::impls::time::TimeImpl;
use rufi::distributed::platform::sync::RuFiPlatform;
use rufi::programs::gradient;
use rumqttc::MqttOptions;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use rufi::core::export::Export;

#[derive(Debug, Default)]
struct Arguments {
    pub num_cycles: i32,
    pub id: i32,
    pub source: bool,
}

impl Arguments {
    pub fn parse<S: AsRef<str>>(args: impl IntoIterator<Item = S>) -> Result<Arguments, String> {
        let mut r = Arguments::default();

        for arg in args {
            match arg.as_ref() {
                "-l" => r.num_cycles = 500,
                "-s" => r.num_cycles = 100,
                "-m" => r.num_cycles = 300,
                "-t" => r.source = true,
                "-f" => r.source = false,
                x => r.id = x.parse::<i32>().unwrap(),
            }
        }

        Ok(r)
    }
}

struct MockDiscovery(i32);

impl Discovery for MockDiscovery {
    fn discover_neighbors(&self) -> Vec<i32> {
        let self_id = self.0;
        vec![self_id - 1, self_id, self_id + 1]
            .into_iter()
            .filter(|n| (n > &0 && n < &6))
            .collect()
    }
}

struct MockSetup;

impl NbrSensorSetup for MockSetup {
    fn nbr_sensor_setup(&self, _nbrs: Vec<i32>) -> NbrSensors {
        Default::default()
    }
}

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    env_logger::init();

    // Get arguments from the CLI
    let args = Arguments::parse(std::env::args().skip(1))?;
    let self_id = args.id;
    let is_source = args.source;
    let num_cycles = args.num_cycles;

    /* Set up a simple topology that will be used for these tests.
     *  Topology: [1] -- [2] -- [3] -- [4] -- [5].
     */
    let discovery = MockDiscovery(self_id);
    let nbrs = discovery.discover_neighbors();

    let setup = MockSetup {};

    // Setup the context
    let local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>> = vec![(
        sensor("source"),
        Rc::new(Box::new(is_source) as Box<dyn Any>),
    )]
    .into_iter()
    .collect();
    let context = Context::new(
        self_id,
        local_sensor.clone(),
        Default::default(),
        Default::default(),
    );

    // Setup the MQTT client network
    let mut mqttoptions =
        MqttOptions::new(format!("device#{}", self_id), "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let network = SyncMQTTNetwork::new(mqttoptions, nbrs.clone(), 10);

    let time = TimeImpl::new();

    let debug_hook = |_: &Export| {
        //println!("EXPORT: {:?}\n OUTPUT:{:?}", export, export.root());
    };
    // Setup the platform and run the program
    RuFiPlatform::new(network?, context, discovery, setup, time, vec![debug_hook])
        .run_n_cycles(gradient, num_cycles as usize)
}
