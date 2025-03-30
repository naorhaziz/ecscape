mod config;
mod ecs_agent_metadata;
mod ecscape;

use anyhow::Result;
use config::{ARCH, VERSION};
use ecscape::ECSCape;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("starting ecscape (version: {}, arch: {})", *VERSION, *ARCH);

    let ecscape = ECSCape::try_new().await?;

    ecscape
        .start()
        .await
        .inspect_err(|err| error!("an error has occured: {err:?}"))
        .inspect(|_| info!("ecsape finished gracefully"))
}
