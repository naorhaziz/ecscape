use crate::acs_client::ACSClient;
use anyhow::Result;
use std::time::Duration;
use tokio_retry2::{Retry, RetryError, strategy::FixedInterval};
use tracing::{debug, error, info, warn};

pub struct ECScape {}

impl ECScape {
    pub fn new() -> Self {
        Self {}
    }

    async fn start_inner() -> Result<()> {
        let mut acs_client = ACSClient::connect().await?;
        debug!("Successfully connected to ACS");

        loop {
            match acs_client.receive().await {
                Ok(Some(message)) => {
                    info!("Received ACS message: {:?}", message);
                }
                Ok(None) => {
                    warn!("ACS connection closed by server");
                    return Err(anyhow::anyhow!("Connection closed by server"));
                }
                Err(err) => {
                    error!("Failed to receive message from ACS: {:?}", err);
                    return Err(err);
                }
            }
        }
    }

    pub async fn start(&self) -> Result<()> {
        const MAX_RETRIES: usize = 5;
        const RETRY_DELAY: Duration = Duration::from_secs(5);

        let retry_strategy =
            FixedInterval::from_millis(RETRY_DELAY.as_millis() as u64).take(MAX_RETRIES);
        Retry::spawn(retry_strategy, || async {
            Self::start_inner().await.map_err(RetryError::transient)
        })
        .await?;

        Ok(())
    }
}
