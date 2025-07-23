use crate::protocols::protocol_client::ProtocolClient;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;
use tokio_retry2::{Retry, RetryError, strategy::ExponentialBackoff};
use tokio_tungstenite::tungstenite::http::Request;
use tracing::{error, warn};

#[async_trait]
pub trait ProtocolHandler<T>: Send + Sync
where
    T: Serialize + DeserializeOwned + Send,
{
    fn build_request(&self) -> Result<Request<()>>;
    async fn handle_message(&self, client: &mut ProtocolClient<T>, message: T) -> Result<()>;

    async fn start_inner(&self) -> Result<()> {
        let request = self.build_request()?;
        let mut client = ProtocolClient::<T>::connect(request).await?;

        loop {
            match client.receive().await {
                Ok(Some(message)) => {
                    if let Err(err) = self.handle_message(&mut client, message).await {
                        warn!("Failed to handle message: {:?}", err);
                        return Err(err);
                    }
                }
                Ok(None) => {
                    warn!("WebSocket connection closed by peer");
                    return Err(anyhow::anyhow!("WebSocket connection closed"));
                }
                Err(err) => {
                    error!("Failed to receive message: {:?}", err);
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
