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
        P: Fn(RoundVM) -> (RoundVM, A) + Copy,
        A: Clone + 'static + FromStr + Display,
    {
        loop {
            // STEP 1: Discover neighbours
            let nbrs = self.discovery.discover_neighbors();
            // STEP 2: Subscribe to the topics of the neighbours
            let subscriptions: Vec<i32> = nbrs
                .clone()
                .into_iter()
                .filter(|n| !self.discovered_nbrs.contains(n))
                .collect();
            self.discovered_nbrs.extend(subscriptions);

            single_cycle(
                &mut self.mailbox,
                &mut self.network,
                &self.nbr_sensor_setup,
                self.context.clone(),
                program,
            )?;
        }
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
    P: Fn(RoundVM) -> (RoundVM, A),
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
    let (mut vm_, result) = round(vm, program);
    let self_export: Export = vm_.export_data().clone();
    println!("OUTPUT: {}\nEXPORT: {}\n", result, self_export);

    //STEP 5: Publish the export
    let msg = Message::new(*vm_.self_id(), self_export, std::time::SystemTime::now());
    let msg_ser = serde_json::to_string(&msg).unwrap();
    network.send(*vm_.self_id(), msg_ser)?;

    //STEP 6: Receive the neighbouring exports from the network
    if let Ok(NetworkUpdate::Update { msg }) = network.receive() {
        if let Ok(msg) = serde_json::from_str(&msg) {
            mailbox.enqueue(msg);
        }
    }
    Ok(())
}
