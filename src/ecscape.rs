use crate::{
    acs_client::ACSClient,
    structs::{
        AckRequestStruct, HeartbeatAckRequestStruct, IAMRoleCredentialsAckRequestStruct,
        ProtocolMessage, RefreshCredentialsAckRequestStruct, TaskStopVerificationAckStruct,
    },
};
use anyhow::Result;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio_retry2::{Retry, RetryError, strategy::FixedInterval};
use tracing::{debug, error, info, warn};

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
                info!("Processing HeartbeatMessage: {:?}", msg);
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

            ProtocolMessage::PayloadMessage(msg) => {
                info!("Processing PayloadMessage: {:?}", msg);
                let payload_ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ProtocolMessage::AckRequest(payload_ack);
                acs_client.send(&ack_message).await?;
                debug!(
                    "Sent PayloadMessage AckRequest for message ID: {}",
                    msg.message_id
                );

                if let Some(tasks) = &msg.tasks {
                    for task in tasks {
                        if let Some(credentials) = &task.role_credentials {
                            let creds_ack = IAMRoleCredentialsAckRequestStruct {
                                message_id: msg.message_id.clone(),
                                credentials_id: credentials.credentials_id.clone(),
                                expiration: credentials.expiration.clone(),
                            };
                            let creds_ack_message =
                                ProtocolMessage::IAMRoleCredentialsAckRequest(creds_ack);
                            acs_client.send(&creds_ack_message).await?;
                            info!(
                                "Sent IAMRoleCredentialsAckRequest for task {}",
                                task.arn.as_ref().unwrap_or(&"unknown".to_string())
                            );
                        }
                    }
                }
            }

            ProtocolMessage::AttachTaskNetworkInterfacesMessage(msg) => {
                info!("Processing AttachTaskNetworkInterfacesMessage: {:?}", msg);
                let ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ProtocolMessage::AckRequest(ack);
                acs_client.send(&ack_message).await?;
                debug!(
                    "Sent AttachTaskNetworkInterfacesMessage AckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ProtocolMessage::AttachInstanceNetworkInterfacesMessage(msg) => {
                info!(
                    "Processing AttachInstanceNetworkInterfacesMessage: {:?}",
                    msg
                );
                let ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ProtocolMessage::AckRequest(ack);
                acs_client.send(&ack_message).await?;
                debug!(
                    "Sent AttachInstanceNetworkInterfacesMessage AckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ProtocolMessage::ConfirmAttachmentMessage(msg) => {
                info!("Processing ConfirmAttachmentMessage: {:?}", msg);
                let ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ProtocolMessage::AckRequest(ack);
                acs_client.send(&ack_message).await?;
                debug!(
                    "Sent ConfirmAttachmentMessage AckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ProtocolMessage::TaskManifestMessage(msg) => {
                info!("Processing TaskManifestMessage: {:?}", msg);
                let ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ProtocolMessage::AckRequest(ack);
                acs_client.send(&ack_message).await?;
                debug!(
                    "Sent TaskManifestMessage AckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ProtocolMessage::IAMRoleCredentialsMessage(msg) => {
                info!("Processing IAMRoleCredentialsMessage: {:?}", msg);
                if let Some(credentials) = &msg.role_credentials {
                    let ack = IAMRoleCredentialsAckRequestStruct {
                        message_id: msg.message_id.clone(),
                        credentials_id: credentials.credentials_id.clone(),
                        expiration: credentials.expiration.clone(),
                    };
                    let ack_message = ProtocolMessage::IAMRoleCredentialsAckRequest(ack);
                    acs_client.send(&ack_message).await?;
                    info!(
                        "Sent IAMRoleCredentialsAckRequest for message ID: {}",
                        msg.message_id
                    );
                } else {
                    warn!(
                        "IAMRoleCredentialsMessage missing credentials for message ID: {}",
                        msg.message_id
                    );
                }
            }

            ProtocolMessage::RefreshCredentialsMessage(msg) => {
                info!("Processing RefreshCredentialsMessage: {:?}", msg);
                if let Some(credentials) = &msg.role_credentials {
                    let ack = RefreshCredentialsAckRequestStruct {
                        message_id: msg.message_id.clone(),
                        task_arn: msg.task_arn.clone(),
                        expiration: credentials.expiration.clone(),
                        credentials_id: credentials.credentials_id.clone(),
                    };
                    let ack_message = ProtocolMessage::RefreshCredentialsAckRequest(ack);
                    acs_client.send(&ack_message).await?;
                    info!(
                        "Sent RefreshCredentialsAckRequest for message ID: {}",
                        msg.message_id
                    );
                } else {
                    warn!(
                        "RefreshCredentialsMessage missing credentials for message ID: {}",
                        msg.message_id
                    );
                }
            }

            ProtocolMessage::TaskStopVerificationMessage(msg) => {
                info!("Processing TaskStopVerificationMessage: {:?}", msg);
                let ack = TaskStopVerificationAckStruct {
                    message_id: msg.message_id.clone(),
                    generated_at: Some(chrono::Utc::now().timestamp_millis()),
                    stop_tasks: msg.stop_candidates.clone(),
                };
                let ack_message = ProtocolMessage::TaskStopVerificationAck(ack);
                acs_client.send(&ack_message).await?;
                debug!(
                    "Sent TaskStopVerificationAck for message ID: {}",
                    msg.message_id
                );
            }

            // These are responses/acks that we send, not messages we should respond to
            ProtocolMessage::HeartbeatAckRequest(_)
            | ProtocolMessage::AckRequest(_)
            | ProtocolMessage::IAMRoleCredentialsAckRequest(_)
            | ProtocolMessage::RefreshCredentialsAckRequest(_)
            | ProtocolMessage::PublishMetricsRequest(_)
            | ProtocolMessage::PublishInstanceStatusRequest(_)
            | ProtocolMessage::TaskStopVerificationAck(_) => {
                debug!("Received response/ack message - no action needed");
            }

            ProtocolMessage::ErrorMessage(msg) => {
                warn!(
                    "Received ErrorMessage from ACS: message_id={}, error_type={:?}, error_message={:?}",
                    msg.message_id, msg.error_type, msg.error_message
                );
            }

            ProtocolMessage::CloseMessage(msg) => {
                warn!(
                    "Received CloseMessage from ACS: message_id={}, reason={:?}",
                    msg.message_id, msg.reason
                );
                return Err(anyhow::anyhow!("ACS sent close message: {:?}", msg.reason));
            }
        }

        Ok(())
    }
}
