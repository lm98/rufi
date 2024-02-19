use crate::discovery::nbr_sensors_setup::NbrSensorSetup;
use crate::discovery::Discovery;
use crate::mailbox::{AsStates, Mailbox};
use crate::message::Message;
use crate::network::{sync::Network, NetworkUpdate};
use crate::time::Time;
use bytes::Bytes;
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::vm::round_vm::RoundVM;
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

/// This struct represents the platform on which the program is executed
pub struct RuFiPlatform<M, N, D, S, T, H>
    where
        M: Mailbox,
        N: Network,
        D: Discovery,
        S: NbrSensorSetup,
        T: Time,
        H: Fn(&Export) -> (),
{
    mailbox: M,
    network: N,
    context: Context,
    discovery: D,
    discovered_nbrs: Vec<i32>,
    nbr_sensor_setup: S,
    time: T,
    hooks: Vec<H>,
}

impl<M, N, D, S, T, H> RuFiPlatform<M, N, D, S, T, H>
    where
        M: Mailbox,
        N: Network,
        D: Discovery,
        S: NbrSensorSetup,
        T: Time,
        H: Fn(&Export) -> (),
{
    /// Creates a new platform
    pub fn new(mailbox: M, network: N, context: Context, discovery: D, setup: S, time: T, hooks: Vec<H>) -> Self {
        RuFiPlatform {
            mailbox,
            network,
            context,
            discovery,
            discovered_nbrs: vec![],
            nbr_sensor_setup: setup,
            time,
            hooks,
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
    pub fn run_forever<P, A>(mut self, program: P) -> Result<(), Box<dyn Error>>
        where
            P: Fn(&mut RoundVM) -> A + Copy,
            A: Clone + 'static + FromStr + Display,
    {
        loop {
            let export = self.single_cycle(program)?;

            for hook in self.hooks.iter() {
                hook(&export);
            }
            //sleep for one sec
            self.time.sleep(Duration::from_secs(1));
        }
    }

    pub fn run_n_cycles<P, A>(mut self, program: P, n: usize) -> Result<(), Box<dyn Error>>
        where
            P: Fn(&mut RoundVM) -> A + Copy,
            A: Clone + 'static + FromStr + Display,
    {
        for _ in 0..n {
            self.single_cycle(program)?;
        }
        Ok(())
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
    fn single_cycle<P, A>(
        &mut self,
        program: P,
    ) -> Result<Export, Box<dyn Error>>
        where
            P: Fn(&mut RoundVM) -> A,
            A: Clone + 'static + FromStr + Display,
            M: Mailbox,
            N: Network,
            S: NbrSensorSetup,
    {
        // STEP 1: Discover neighbours
        let nbrs = self.discovery.discover_neighbors();
        // STEP 2: Subscribe to the topics of the neighbours
        let subscriptions: Vec<i32> = nbrs
            .clone()
            .into_iter()
            .filter(|n| !self.discovered_nbrs.contains(n))
            .collect();
        self.discovered_nbrs.extend(subscriptions);

        //STEP 3: Retrieve the neighbouring exports from the mailbox
        let states = self.mailbox.messages().as_states();

        //STEP 4: Execute a round
        let nbr_sensors = self.nbr_sensor_setup.nbr_sensor_setup(states.keys().cloned().collect());
        let context = Context::new(
            *self.context.self_id(),
            self.context.local_sensors().clone(),
            nbr_sensors,
            states,
        );
        //println!("CONTEXT: {:?}", context);
        let mut vm = RoundVM::new(context);
        vm.new_export_stack();
        let result = round(&mut vm, program);
        let self_export: Export = vm.export_data().clone();
        println!("OUTPUT: {}\nEXPORT: {}\n", result, self_export);

        //STEP 5: Publish the export
        let msg = Message::new(*vm.self_id(), self_export.clone(), std::time::SystemTime::now());
        if let Ok(msg_ser) = serde_json::to_vec(&msg) {
            if let Err(e) = self.network.send(*vm.self_id(), Bytes::from(msg_ser)) {
                println!("Error sending the message: {}", e);
            }
        } else {
            println!("Error while serializing the message");
        }

        //STEP 6: Receive the neighbouring exports from the network
        match self.network.receive() {
            Ok(NetworkUpdate::Update { msg }) => {
                if let Ok(msg) = serde_json::from_slice(&msg) {
                    self.mailbox.enqueue(msg);
                    Ok(self_export)
                } else {
                    Err("Error deserializing the message".into())
                }
            }
            Ok(NetworkUpdate::None) => {
                println!("No message received from the network");
                Ok(self_export)
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
}

