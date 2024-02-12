use crate::discovery::nbr_sensors_setup::NbrSensorSetup;
use crate::discovery::Discovery;
use crate::mailbox::{AsStates, Mailbox};
use crate::message::Message;
use crate::network::{asynchronous::Network, NetworkUpdate};
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::vm::round_vm::RoundVM;
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;
use bytes::Bytes;
use crate::time::Time;

/// This struct represents the platform on which the program is executed
pub struct RuFiPlatform<M, N, D, S, T>
where
    M: Mailbox,
    N: Network,
    D: Discovery,
    S: NbrSensorSetup,
    T: Time,
{
    mailbox: M,
    network: N,
    context: Context,
    discovery: D,
    discovered_nbrs: Vec<i32>,
    nbr_sensor_setup: S,
    time: T
}

impl<M, N, D, S, T> RuFiPlatform<M, N, D, S, T>
where
    M: Mailbox,
    N: Network,
    D: Discovery,
    S: NbrSensorSetup,
    T: Time,
{
    /// Creates a new platform
    pub fn new(mailbox: M, network: N, context: Context, discovery: D, setup: S, time: T) -> Self {
        RuFiPlatform {
            mailbox,
            network,
            context,
            discovery,
            discovered_nbrs: vec![],
            nbr_sensor_setup: setup,
            time,
        }
    }

    /// Runs indefinitely the program on the platform
    ///
    /// # Arguments
    ///
    /// * `program` - The aggregate program to be executed
    ///
    /// # Generic Arguments
    ///
    /// * `P` - The type of the aggregate program, it must be a function that takes a [RoundVM] and returns a [RoundVM] and a result of type `A`
    /// * `A` - The type of the result of the aggregate program
    pub async fn run_forever<P, A>(mut self, program: P) -> Result<(), Box<dyn Error>>
    where
        P: Fn(&mut RoundVM) -> A + Copy,
        A: Clone + 'static + FromStr + Display,
    {
        loop {
            self.pre_cycle();
            single_cycle(
                &mut self.mailbox,
                &mut self.network,
                &self.nbr_sensor_setup,
                self.context.clone(),
                program,
            ).await?;
            println!("Cycle ended, starting a new cycle");
            //sleep for one sec
            self.time.sleep(Duration::from_secs(1)).await
        }
    }

    pub async fn run_n_cycles<P, A>(mut self, program: P, n: usize) -> Result<(), Box<dyn Error>>
    where
        P: Fn(&mut RoundVM) -> A + Copy,
        A: Clone + 'static + FromStr + Display,
    {
        for _ in 0..n {
            self.pre_cycle();

            single_cycle(
                &mut self.mailbox,
                &mut self.network,
                &self.nbr_sensor_setup,
                self.context.clone(),
                program,
            ).await?;
        }
        Ok(())
    }

    /// Performs the pre-cycle operations
    fn pre_cycle(&mut self) {
        // STEP 1: Discover neighbours
        let nbrs = self.discovery.discover_neighbors();
        // STEP 2: Subscribe to the topics of the neighbours
        let subscriptions: Vec<i32> = nbrs
            .clone()
            .into_iter()
            .filter(|n| !self.discovered_nbrs.contains(n))
            .collect();
        self.discovered_nbrs.extend(subscriptions);
    }
}

/// Performs a single step of the execution cycle of an aggregate program
///
/// # Arguments
///
/// * `mailbox` - The mailbox of the device
/// * `network` - The network through which the device communicates
/// * `context` - The context of the device
/// * `program` - The aggregate program to be executed
///
/// # Generic Arguments
///
/// * `P` - The type of the aggregate program, it must be a function that takes a [RoundVM] and returns a [RoundVM] and a result of type `A`
/// * `A` - The type of the result of the aggregate program
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - The result of the execution
async fn single_cycle<P, A, M, N, S>(
    mailbox: &mut M,
    network: &mut N,
    setup: &S,
    context: Context,
    program: P,
) -> Result<(), Box<dyn Error>>
where
    P: Fn(&mut RoundVM) -> A,
    A: Clone + 'static + FromStr + Display,
    M: Mailbox,
    N: Network,
    S: NbrSensorSetup,
{
    //STEP 3: Retrieve the neighbouring exports from the mailbox
    let states = mailbox.messages().as_states();

    //STEP 4: Execute a round
    let nbr_sensors = setup.nbr_sensor_setup(states.keys().cloned().collect());
    let context = Context::new(
        *context.self_id(),
        context.local_sensors().clone(),
        nbr_sensors,
        states,
    );
    println!("CONTEXT: {:?}", context);
    let mut vm = RoundVM::new(context);
    vm.new_export_stack();
    let result = round(&mut vm, program);
    let self_export: Export = vm.export_data().clone();
    println!("OUTPUT: {}\nEXPORT: {}\n", result, self_export);

    //STEP 5: Publish the export
    let msg = Message::new(*vm.self_id(), self_export, std::time::SystemTime::now());
    if let Ok(msg_ser) = serde_json::to_vec(&msg) {
        network.send(*vm.self_id(), Bytes::from(msg_ser)).await?;
    } else {
        println!("Error while serializing the message");
    }

    //STEP 6: Receive the neighbouring exports from the network
    match network.receive().await {
        Ok(NetworkUpdate::Update { msg }) => {
            if let Ok(msg) = serde_json::from_slice(&msg) {
                mailbox.enqueue(msg);
                Ok(())
            } else {
                Err("Error deserializing the message".into())
            }
        }
        Ok(NetworkUpdate::None) => {
            println!("No message received from the network");
            Ok(())
        }
        Ok(NetworkUpdate::Err { reason }) => {
            println!("Error receiving from the network: {}", reason);
            Err(reason.into())
        }
        _ => {
            println!("Error receiving from the network");
            Err("Error receiving from the network".into())
        }
    }
}
