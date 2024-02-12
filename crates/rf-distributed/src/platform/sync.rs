use crate::discovery::nbr_sensors_setup::NbrSensorSetup;
use crate::discovery::Discovery;
use crate::mailbox::{AsStates, Mailbox};
use crate::message::Message;
use crate::network::{sync::Network, NetworkUpdate};
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::vm::round_vm::RoundVM;
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
pub struct SyncRuFiPlatform<M, N, D, S>
where
    M: Mailbox,
    N: Network,
    D: Discovery,
    S: NbrSensorSetup,
{
    mailbox: M,
    network: N,
    context: Context,
    discovery: D,
    discovered_nbrs: Vec<i32>,
    nbr_sensor_setup: S,
}

impl<M, N, D, S> SyncRuFiPlatform<M, N, D, S>
where
    M: Mailbox,
    N: Network,
    D: Discovery,
    S: NbrSensorSetup,
{
    pub fn new(mailbox: M, network: N, context: Context, discovery: D, setup: S) -> Self {
        SyncRuFiPlatform {
            mailbox,
            network,
            context,
            discovery,
            discovered_nbrs: vec![],
            nbr_sensor_setup: setup,
        }
    }

    pub fn run_forever<P, A>(mut self, program: P) -> Result<(), Box<dyn Error>>
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
            )?;
        }
    }

    pub fn run_n_cycles<P, A>(mut self, program: P, n: usize) -> Result<(), Box<dyn Error>>
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
            )?;
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

fn single_cycle<P, A, M, N, S>(
    mailbox: &mut M,
    network: &mut N,
    setup: &S,
    context: Context,
    program: P,
) -> Result<(), Box<dyn Error>>
where
    P: Fn(&mut RoundVM) -> A + Copy,
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
    if let Ok(msg) = serde_json::to_string(&msg) {
        network.send(*vm.self_id(), msg.into())?;
    } else {
        println!("Could not serialize the message");
    }

    //STEP 6: Receive the neighbouring exports from the network
    if let Ok(NetworkUpdate::Update { msg }) = network.receive() {
        if let Ok(msg) = serde_json::from_slice(&msg) {
            mailbox.enqueue(msg);
        }
    }
    Ok(())
}
