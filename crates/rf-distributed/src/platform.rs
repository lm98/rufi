use crate::discovery::nbr_sensors_setup::NbrSensorSetup;
use crate::discovery::Discovery;
use crate::mailbox::Mailbox;
use rf_core::context::Context;

pub mod asynchronous;
pub mod sync;

pub struct PlatformFactory;

impl PlatformFactory {
    pub fn sync_platform<M, N, D, S>(
        mailbox: M,
        network: N,
        context: Context,
        discovery: D,
        setup: S,
    ) -> sync::SyncRuFiPlatform<M, N, D, S>
    where
        M: Mailbox,
        N: crate::network::sync::Network,
        D: Discovery,
        S: NbrSensorSetup,
    {
        sync::SyncRuFiPlatform::new(mailbox, network, context, discovery, setup)
    }

    pub fn async_platform<M, N, D, S>(
        mailbox: M,
        network: N,
        context: Context,
        discovery: D,
        setup: S,
    ) -> asynchronous::RuFiPlatform<M, N, D, S>
    where
        M: Mailbox,
        N: crate::network::asynchronous::Network,
        D: Discovery,
        S: NbrSensorSetup,
    {
        asynchronous::RuFiPlatform::new(mailbox, network, context, discovery, setup)
    }
}
