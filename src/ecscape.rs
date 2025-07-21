use crate::{
    acs_client::ACSClient,
    structs::{HeartbeatAckRequestStruct, ProtocolMessage},
};
use anyhow::Result;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio_retry2::{Retry, RetryError, strategy::FixedInterval};
use tracing::{debug, error, warn};

pub struct ECScape {}

impl ECScape {
    pub fn new() -> Self {
        Self {}
    }

    async fn start_inner(send_credentials: bool) -> Result<()> {
        let mut acs_client = ACSClient::connect(send_credentials).await?;
        debug!("Successfully connected to ACS");

        loop {
            match acs_client.receive().await {
                Ok(Some(message)) => {
                    if let Err(err) = Self::handle_message(&mut acs_client, &message).await {
                        warn!("Failed to handle ACS message: {:?}", err);
                        return Err(err);
                    }
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
        const RETRY_DELAY: Duration = Duration::from_secs(5);

        let retry_strategy = FixedInterval::from_millis(RETRY_DELAY.as_millis() as u64);

        let first_attempt = AtomicBool::new(true);
        Retry::spawn(retry_strategy, || async {
            let send_credentials = first_attempt.swap(false, Ordering::Relaxed);
            Self::start_inner(send_credentials)
                .await
                .map_err(RetryError::transient)
        })
        .await?;

        Ok(())
    }

    async fn handle_message(acs_client: &mut ACSClient, message: &ProtocolMessage) -> Result<()> {
        match message {
            ProtocolMessage::HeartbeatMessage(msg) => {
                debug!("Processing HeartbeatMessage with ID: {}", msg.message_id);
                let heartbeat_ack = HeartbeatAckRequestStruct {
                    message_id: msg.message_id.clone(),
                };
                let ack_message = ProtocolMessage::HeartbeatAckRequest(heartbeat_ack);
                acs_client.send(&ack_message).await?;
                debug!(
                    "Sent HeartbeatAckRequest for message ID: {}",
                    msg.message_id
                );
            }

            _ => {
                debug!("Received ACS message: {:?}", message);
            }
        }

        Ok(())
    }
}
