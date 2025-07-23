pub mod acs_handler;
pub mod tcs_handler;

use crate::protocols::{ecs_protocol_client::ECSProtocolClient, structs::ProtocolMessage};
use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio_retry2::{Retry, RetryError, strategy::ExponentialBackoff};
use tokio_tungstenite::tungstenite::http::Request;
use tracing::{error, warn};

#[async_trait]
pub trait ECSProtocolHandler: Send + Sync {
    fn build_request(&self) -> Result<Request<()>>;
    async fn handle_message(
        &self,
        ecs_protocol_client: &mut ECSProtocolClient,
        message: ProtocolMessage,
    ) -> Result<()>;

    async fn start_inner(&self) -> Result<()> {
        let request = self.build_request()?;
        let mut ecs_protocol_client = ECSProtocolClient::connect(request).await?;

        loop {
            match ecs_protocol_client.receive().await {
                Ok(Some(message)) => {
                    if let Err(err) = self.handle_message(&mut ecs_protocol_client, message).await {
                        warn!("Failed to handle ECS message: {:?}", err);
                        return Err(err);
                    }
                }
                Ok(None) => {
                    warn!("ECS connection closed by server");
                    return Err(anyhow::anyhow!("Connection closed by server"));
                }
                Err(err) => {
                    error!("Failed to receive message from ECS: {:?}", err);
                    return Err(err);
                }
            }
        }
    }

    async fn start(&self) -> Result<()> {
        // Official ECS Agent reconnection timing constants
        const CONNECTION_BACKOFF_MIN: Duration = Duration::from_millis(250);
        const CONNECTION_BACKOFF_MAX: Duration = Duration::from_secs(120);
        const CONNECTION_BACKOFF_MULTIPLIER: u64 = 2;

        let retry_strategy =
            ExponentialBackoff::from_millis(CONNECTION_BACKOFF_MIN.as_millis() as u64)
                .max_delay(CONNECTION_BACKOFF_MAX)
                .factor(CONNECTION_BACKOFF_MULTIPLIER);

        Retry::spawn(retry_strategy, || async {
            self.start_inner().await.map_err(RetryError::transient)
        })
        .await?;

        Ok(())
    }
}
