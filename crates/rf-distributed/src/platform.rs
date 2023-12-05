use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::vm::round_vm::RoundVM;
use crate::discovery::Discovery;
use crate::mailbox::{AsStates, Mailbox};
use crate::message::Message;
use crate::network::{Network, NetworkUpdate};

/// This struct represents the platform on which the program is executed
pub struct RuFiPlatform {
    mailbox: Box<dyn Mailbox>,
    network: Box<dyn Network>,
    context: Context,
    discovery: Box<dyn Discovery>,
    discovered_nbrs: Vec<i32>,
}

impl RuFiPlatform {
    /// Creates a new platform
    pub fn new(mailbox: Box<dyn Mailbox>, network: Box<dyn Network>, context: Context, discovery: Box<dyn Discovery>) -> Self {
        RuFiPlatform {
            mailbox,
            network,
            context,
            discovery,
            discovered_nbrs: vec![],
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
            P: Fn(RoundVM) -> (RoundVM, A) + Copy,
            A: Clone + 'static + FromStr + Display,
    {
        loop {
            // STEP 1: Discover neighbours
            let nbrs = self.discovery.discover_neighbors();
            // STEP 2: Subscribe to the topics of the neighbours
            let subscriptions : Vec<i32> = nbrs.into_iter().filter(|n| !self.discovered_nbrs.contains(n)).collect();
            self.network.subscribe(subscriptions.clone()).await?;
            self.discovered_nbrs.extend(subscriptions);

            single_cycle(&mut self.mailbox, &mut self.network, self.context.clone(), program).await?;

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
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
async fn single_cycle<P, A>(mailbox: &mut Box<dyn Mailbox>, network: &mut Box<dyn Network>, context: Context, program: P) -> Result<(), Box<dyn Error>>
    where
        P: Fn(RoundVM) -> (RoundVM, A),
        A: Clone + 'static + FromStr + Display,
{
    //STEP 3: Retrieve the neighbouring exports from the mailbox
    let states = mailbox.messages().as_states();

    //STEP 4: Execute a round
    let context = Context::new(
        context.self_id().clone(),
        context.local_sensors().clone(),
        context.nbr_sensors().clone(),
        states,
    );
    println!("CONTEXT: {:?}", context);
    let mut vm = RoundVM::new(context);
    vm.new_export_stack();
    let (mut vm_, result) = round(vm, program);
    let self_export: Export = vm_.export_data().clone();
    println!("OUTPUT: {}\nEXPORT: {}\n", result, self_export);

    //STEP 5: Publish the export
    let msg = Message::new(vm_.self_id().clone(), self_export, std::time::SystemTime::now());
    let msg_ser = serde_json::to_string(&msg).unwrap();
    network.send(vm_.self_id().clone(), msg_ser).await?;

    //STEP 6: Receive the neighbouring exports from the network
    if let Ok(update) = network.recv().await {
        match update {
            NetworkUpdate::Update { msg } => {
                let msg: Message = serde_json::from_str(&msg).unwrap();
                mailbox.enqueue(msg);
            }
            _ => {}
        }
    }
    Ok(())
}