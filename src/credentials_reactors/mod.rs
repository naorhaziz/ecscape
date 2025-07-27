use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast::{Receiver, error::RecvError};
use tracing::{debug, error, warn};

pub mod s3_reactor;
pub mod secrets_manager_reactor;

#[derive(Clone, Debug)]
pub struct ECScapeCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
}

#[async_trait]
pub trait CredentialsReactor {
    async fn react(&self, credentials: ECScapeCredentials) -> Result<()>;

    async fn start(&self, mut credentials_receiver: Receiver<ECScapeCredentials>) -> Result<()> {
        loop {
            match credentials_receiver.recv().await {
                Ok(credentials) => {
                    if let Err(err) = self.react(credentials).await {
                        debug!("Credentials reactor failed to react: {:?}", err);
                    }
                }
                Err(RecvError::Lagged(skipped)) => {
                    warn!(
                        "Credentials reactor lagged behind, skipped {} messages",
                        skipped
                    );
                }
                Err(RecvError::Closed) => {
                    error!("Credentials broadcast channel closed, stopping reactor");
                    break;
                }
            }
        }
        Ok(())
    }
}
