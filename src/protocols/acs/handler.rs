use crate::protocols::{
    acs::{
        request_builder::ACSRequestBuilder,
        structs::{
            ACSMessage, AckRequestStruct, HeartbeatAckRequestStruct,
            IAMRoleCredentialsAckRequestStruct, RefreshCredentialsAckRequestStruct,
            TaskStopVerificationAckStruct,
        },
    },
    protocol_client::ProtocolClient,
    protocol_handler::ProtocolHandler,
    request_builder::RequestBuilder,
};
use anyhow::Result;
use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sdk_ecs::operation::discover_poll_endpoint::DiscoverPollEndpointOutput;
use chrono;
use tokio_tungstenite::tungstenite::http::Request;
use tracing::{debug, info, warn};

pub struct ACSHandler {
    request_builder: ACSRequestBuilder,
    credentials: Credentials,
    discover_poll_endpoint_output: DiscoverPollEndpointOutput,
    region: String,
    cluster_arn: String,
    container_instance_arn: String,
    agent_version: String,
    agent_hash: String,
}

impl ACSHandler {
    pub fn new(
        credentials: Credentials,
        discover_poll_endpoint_output: DiscoverPollEndpointOutput,
        region: String,
        cluster_arn: String,
        container_instance_arn: String,
        agent_version: String,
        agent_hash: String,
    ) -> Self {
        Self {
            request_builder: ACSRequestBuilder::new(),
            credentials,
            discover_poll_endpoint_output,
            region,
            cluster_arn,
            container_instance_arn,
            agent_version,
            agent_hash,
        }
    }
}

#[async_trait]
impl ProtocolHandler<ACSMessage> for ACSHandler {
    fn build_request(&self) -> Result<Request<()>> {
        self.request_builder.build_request(
            self.credentials.clone(),
            self.discover_poll_endpoint_output.clone(),
            &self.region,
            &self.cluster_arn,
            &self.container_instance_arn,
            &self.agent_version,
            &self.agent_hash,
        )
    }

    async fn handle_message(
        &self,
        client: &mut ProtocolClient<ACSMessage>,
        message: ACSMessage,
    ) -> Result<()> {
        match message {
            ACSMessage::HeartbeatMessage(msg) => {
                info!("Processing HeartbeatMessage: {:?}", msg);
                let heartbeat_ack = HeartbeatAckRequestStruct {
                    message_id: msg.message_id.clone(),
                };
                let ack_message = ACSMessage::HeartbeatAckRequest(heartbeat_ack);
                client.send(&ack_message).await?;
                debug!(
                    "Sent HeartbeatAckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ACSMessage::PayloadMessage(msg) => {
                info!("Processing PayloadMessage: {:?}", msg);
                let payload_ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ACSMessage::AckRequest(payload_ack);
                client.send(&ack_message).await?;
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
                                ACSMessage::IAMRoleCredentialsAckRequest(creds_ack);
                            client.send(&creds_ack_message).await?;
                            info!(
                                "Sent IAMRoleCredentialsAckRequest for task {}",
                                task.arn.as_ref().unwrap_or(&"unknown".to_string())
                            );
                        }
                    }
                }
            }

            ACSMessage::AttachTaskNetworkInterfacesMessage(msg) => {
                info!("Processing AttachTaskNetworkInterfacesMessage: {:?}", msg);
                let ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ACSMessage::AckRequest(ack);
                client.send(&ack_message).await?;
                debug!(
                    "Sent AttachTaskNetworkInterfacesMessage AckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ACSMessage::AttachInstanceNetworkInterfacesMessage(msg) => {
                info!(
                    "Processing AttachInstanceNetworkInterfacesMessage: {:?}",
                    msg
                );
                let ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ACSMessage::AckRequest(ack);
                client.send(&ack_message).await?;
                debug!(
                    "Sent AttachInstanceNetworkInterfacesMessage AckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ACSMessage::ConfirmAttachmentMessage(msg) => {
                info!("Processing ConfirmAttachmentMessage: {:?}", msg);
                let ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ACSMessage::AckRequest(ack);
                client.send(&ack_message).await?;
                debug!(
                    "Sent ConfirmAttachmentMessage AckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ACSMessage::TaskManifestMessage(msg) => {
                info!("Processing TaskManifestMessage: {:?}", msg);
                let ack = AckRequestStruct {
                    message_id: msg.message_id.clone(),
                    cluster: msg.cluster_arn.clone(),
                    container_instance: msg.container_instance_arn.clone(),
                };
                let ack_message = ACSMessage::AckRequest(ack);
                client.send(&ack_message).await?;
                debug!(
                    "Sent TaskManifestMessage AckRequest for message ID: {}",
                    msg.message_id
                );
            }

            ACSMessage::IAMRoleCredentialsMessage(msg) => {
                info!("Processing IAMRoleCredentialsMessage: {:?}", msg);
                if let Some(credentials) = &msg.role_credentials {
                    let ack = IAMRoleCredentialsAckRequestStruct {
                        message_id: msg.message_id.clone(),
                        credentials_id: credentials.credentials_id.clone(),
                        expiration: credentials.expiration.clone(),
                    };
                    let ack_message = ACSMessage::IAMRoleCredentialsAckRequest(ack);
                    client.send(&ack_message).await?;
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

            ACSMessage::RefreshCredentialsMessage(msg) => {
                info!("Processing RefreshCredentialsMessage: {:?}", msg);
                if let Some(credentials) = &msg.role_credentials {
                    let ack = RefreshCredentialsAckRequestStruct {
                        message_id: msg.message_id.clone(),
                        task_arn: msg.task_arn.clone(),
                        expiration: credentials.expiration.clone(),
                        credentials_id: credentials.credentials_id.clone(),
                    };
                    let ack_message = ACSMessage::RefreshCredentialsAckRequest(ack);
                    client.send(&ack_message).await?;
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

            ACSMessage::TaskStopVerificationMessage(msg) => {
                info!("Processing TaskStopVerificationMessage: {:?}", msg);
                let ack = TaskStopVerificationAckStruct {
                    message_id: msg.message_id.clone(),
                    generated_at: Some(chrono::Utc::now().timestamp_millis()),
                    stop_tasks: msg.stop_candidates.clone(),
                };
                let ack_message = ACSMessage::TaskStopVerificationAck(ack);
                client.send(&ack_message).await?;
                debug!(
                    "Sent TaskStopVerificationAck for message ID: {}",
                    msg.message_id
                );
            }

            // These are responses/acks that we send, not messages we should respond to
            ACSMessage::HeartbeatAckRequest(_)
            | ACSMessage::AckRequest(_)
            | ACSMessage::IAMRoleCredentialsAckRequest(_)
            | ACSMessage::RefreshCredentialsAckRequest(_)
            | ACSMessage::PublishMetricsRequest(_)
            | ACSMessage::PublishInstanceStatusRequest(_)
            | ACSMessage::TaskStopVerificationAck(_) => {
                debug!("Received response/ack message - no action needed");
            }

            ACSMessage::ErrorMessage(msg) => {
                warn!(
                    "Received ErrorMessage from ACS: message_id={}, error_type={:?}, error_message={:?}",
                    msg.message_id, msg.error_type, msg.error_message
                );
            }

            ACSMessage::CloseMessage(msg) => {
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
