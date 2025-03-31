mod config;
mod ecs_agent_metadata;
mod ecs_protocol_client;
mod ecscape;
mod imds_metadata;
mod structs;
mod utils;
mod ws_client;

use anyhow::Result;
use clap::Parser;
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

#[derive(Debug, Parser)]
#[command(arg_required_else_help(true))]
struct Command {
    #[arg(long)]
    s3_bucket: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    tracing_subscriber::fmt::init();

    let opts = Command::try_parse()?;

    info!("starting ecscape (version: {}, arch: {})", *VERSION, *ARCH);

    let ecscape = ECSCape::try_new().await?;

    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;

    let res = select! {
        res = ecscape.start(opts.s3_bucket) => res,
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
