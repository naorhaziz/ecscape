mod config;
mod ecs_agent_metadata;
mod ecs_protocol_client;
mod ecscape;
mod imds_metadata;
mod structs;
mod utils;
mod ws_client;

use anyhow::Result;
use config::{ARCH, VERSION};
use ecscape::ECSCape;
use mimalloc::MiMalloc;
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
};
use tracing::{error, info, warn};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("starting ecscape (version: {}, arch: {})", *VERSION, *ARCH);

    let ecscape = ECSCape::try_new().await?;

    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;

    let res = select! {
        res = ecscape.start() => res,
        _ = interrupt.recv() => {
            warn!("SIGINT received, stopping...");
            Ok(())
        }
        _ = terminate.recv() => {
            warn!("SIGTERM received, stopping...");
            Ok(())
        }
    };

    res.inspect_err(|err| error!("an error has occured: {err:?}"))
        .inspect(|_| info!("ecsape finished gracefully"))
}
