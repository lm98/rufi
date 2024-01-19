use rf_core::context::Context;
use crate::discovery::Discovery;
use crate::discovery::nbr_sensors_setup::NbrSensorSetup;
use crate::mailbox::Mailbox;

pub mod sync;
pub mod asynchronous;

pub struct PlatformFactory;

impl PlatformFactory {
    pub fn sync_platform(
        mailbox: Box<dyn Mailbox>,
        network: Box<dyn crate::network::sync::Network>,
        context: Context,
        discovery: Box<dyn Discovery>,
        setup: Box<dyn NbrSensorSetup>,
    ) -> sync::SyncRuFiPlatform {
        sync::SyncRuFiPlatform::new(mailbox, network, context, discovery, setup)
    }

    pub fn async_platform(
        mailbox: Box<dyn Mailbox>,
        network: Box<dyn crate::network::asynchronous::Network>,
        context: Context,
        discovery: Box<dyn Discovery>,
        setup: Box<dyn NbrSensorSetup>,
    ) -> asynchronous::RuFiPlatform {
        asynchronous::RuFiPlatform::new(mailbox, network, context, discovery, setup)
    }
}
