use crate::{
    config::{ARCH, VERSION},
    credentials_reactors::{
        CredentialsReactor, ECScapeCredentials, s3_reactor::S3Reactor,
        secrets_manager_reactor::SecretsManagerReactor,
    },
    ecscape::ECScape,
    imds_metadata::IMDSMetadata,
};
use anyhow::Result;
use clap::Parser;
use std::thread;
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    sync::broadcast,
};
use tracing::{error, info, warn};

mod config;
mod credentials_reactors;
mod ecs_agent_metadata;
mod ecscape;
mod imds_metadata;
mod protocols;
mod ws_client;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser, Debug)]
#[command(arg_required_else_help(false))]
struct Args {
    #[arg(long)]
    s3_bucket: Option<String>,
    #[arg(long)]
    secret_arn: Option<String>,
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

    let imds_metadata = IMDSMetadata::try_new().await?;

    let (credentials_sender, _) = broadcast::channel::<ECScapeCredentials>(1024);

    let s3_reactor = S3Reactor::new(args.s3_bucket, imds_metadata.region.clone());

    let secrets_reactor = SecretsManagerReactor::new(args.secret_arn, imds_metadata.region.clone());

    let ecscape = ECScape::try_new(imds_metadata).await?;

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

        res = s3_reactor.start(credentials_sender.subscribe()) => res,
        res = secrets_reactor.start(credentials_sender.subscribe()) => res,

        res = ecscape.start(credentials_sender) => res,
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

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

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
