use crate::{
    config::{ARCH, VERSION},
    ecscape::ECScape,
};
use anyhow::Result;
use clap::Parser;
use std::thread;
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
};
use tracing::{error, info, warn};

mod config;
mod ecs_agent_metadata;
mod ecscape;
mod imds_metadata;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser, Debug)]
#[command(arg_required_else_help(false))]
struct Args {
    #[arg(long, env = "PORT", default_value = "8080")]
    port: u16,
}

async fn main_inner() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            error!("Failed to parse command line arguments: {:?}", err);
            err.exit();
        }
    };

    info!("ecscape started ({}-{})", VERSION.as_str(), ARCH.as_str());

    let ecscape = ECScape::try_new().await?;

    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;

    let res = select! {
        _ = interrupt.recv() => {
            warn!("SIGINT received, stopping...");
            Ok(())
        }
        _ = terminate.recv() => {
            warn!("SIGTERM received, stopping...");
            Ok(())
        }

        res = ecscape.start() => res,
    };

    res
}

async fn async_main() -> Result<()> {
    main_inner()
        .await
        .inspect(|_| info!("ecscape finished gracefully"))
        .inspect_err(|err| error!("ecscape finished with error: {:?}", err))?;

    Ok(())
}

fn main() -> Result<()> {
    const MAX_THREADS: usize = 8;

    let worker_threads = thread::available_parallelism()
        .map(|o| o.get())
        .unwrap_or(MAX_THREADS)
        .min(MAX_THREADS);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()?
        .block_on(async_main())?;

    Ok(())
}
