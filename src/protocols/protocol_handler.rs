use crate::protocols::protocol_client::ProtocolClient;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;
use tokio_retry2::{Retry, RetryError, strategy::ExponentialBackoff};
use tokio_tungstenite::tungstenite::http::Request;

#[async_trait]
pub trait ProtocolHandler<T>: Send + Sync
where
    T: Serialize + DeserializeOwned + Send,
{
    fn build_request(&self) -> Result<Request<()>>;
    async fn start_inner(&self, client: ProtocolClient<T>) -> Result<()>;

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
            let request = self.build_request()?;
            let client = ProtocolClient::<T>::connect(request).await?;
            self.start_inner(client)
                .await
                .map_err(RetryError::transient)
        })
        .await?;

        Ok(())
    }
}
