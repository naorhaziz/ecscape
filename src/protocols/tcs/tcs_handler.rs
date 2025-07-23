use crate::protocols::{
    ecs_protocol_client::ECSProtocolClient,
    handlers::ECSProtocolHandler,
    request_builders::{RequestBuilder, tcs_request_builder::TCSRequestBuilder},
    structs::{HeartbeatAckRequestStruct, ProtocolMessage},
};
use anyhow::Result;
use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sdk_ecs::operation::discover_poll_endpoint::DiscoverPollEndpointOutput;
use tokio_tungstenite::tungstenite::http::Request;
use tracing::{debug, info, warn};

pub struct TCSHandler {
    request_builder: TCSRequestBuilder,
    credentials: Credentials,
    discover_poll_endpoint_output: DiscoverPollEndpointOutput,
    region: String,
    cluster_arn: String,
    container_instance_arn: String,
    agent_version: String,
    agent_hash: String,
}

impl TCSHandler {
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
            request_builder: TCSRequestBuilder::new(),
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
impl ECSProtocolHandler for TCSHandler {
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
        ecs_protocol_client: &mut ECSProtocolClient,
        message: ProtocolMessage,
    ) -> Result<()> {
        match message {
            // TCS-specific messages
            ProtocolMessage::AckPublishMetric(msg) => {
                info!("Received AckPublishMetric from TCS: {:?}", msg);
                // TCS acknowledges our metrics were received - no action needed
            }

            ProtocolMessage::AckPublishHealth(msg) => {
                info!("Received AckPublishHealth from TCS: {:?}", msg);
                // TCS acknowledges our health metrics were received - no action needed
            }

            ProtocolMessage::AckPublishInstanceStatus(msg) => {
                info!("Received AckPublishInstanceStatus from TCS: {:?}", msg);
                // TCS acknowledges our instance status was received - no action needed
            }

            ProtocolMessage::StopTelemetrySessionMessage(msg) => {
                warn!("Received StopTelemetrySessionMessage from TCS: {:?}", msg);
                return Err(anyhow::anyhow!("TCS requested to stop telemetry session"));
            }

            // Standard heartbeat handling (common between ACS and TCS)
            ProtocolMessage::HeartbeatMessage(msg) => {
                info!("Processing TCS HeartbeatMessage: {:?}", msg);
                let heartbeat_ack = HeartbeatAckRequestStruct {
                    message_id: msg.message_id.clone(),
                };
                let ack_message = ProtocolMessage::HeartbeatAckRequest(heartbeat_ack);
                ecs_protocol_client.send(&ack_message).await?;
                debug!(
                    "Sent HeartbeatAckRequest to TCS for message ID: {}",
                    msg.message_id
                );
            }

            // TCS doesn't typically receive these ACS-style messages, but handle them gracefully
            ProtocolMessage::PayloadMessage(_)
            | ProtocolMessage::TaskManifestMessage(_)
            | ProtocolMessage::AttachTaskNetworkInterfacesMessage(_)
            | ProtocolMessage::AttachInstanceNetworkInterfacesMessage(_)
            | ProtocolMessage::ConfirmAttachmentMessage(_)
            | ProtocolMessage::IAMRoleCredentialsMessage(_)
            | ProtocolMessage::RefreshCredentialsMessage(_)
            | ProtocolMessage::TaskStopVerificationMessage(_) => {
                warn!(
                    "Received unexpected ACS-style message on TCS connection: {:?}",
                    message
                );
                // TCS shouldn't receive these messages - log but don't error
            }

            // These are messages we send, not receive
            ProtocolMessage::HeartbeatAckRequest(_)
            | ProtocolMessage::PublishMetricsRequest(_)
            | ProtocolMessage::PublishHealthRequest(_)
            | ProtocolMessage::PublishInstanceStatusRequest(_)
            | ProtocolMessage::StartTelemetrySessionRequest(_)
            | ProtocolMessage::AckRequest(_)
            | ProtocolMessage::IAMRoleCredentialsAckRequest(_)
            | ProtocolMessage::RefreshCredentialsAckRequest(_)
            | ProtocolMessage::TaskStopVerificationAck(_) => {
                debug!(
                    "Received outbound message type on TCS - ignoring: {:?}",
                    message
                );
            }

            ProtocolMessage::ErrorMessage(msg) => {
                warn!(
                    "Received ErrorMessage from TCS: message_id={}, error_type={:?}, error_message={:?}",
                    msg.message_id, msg.error_type, msg.error_message
                );
            }

            ProtocolMessage::CloseMessage(msg) => {
                warn!(
                    "Received CloseMessage from TCS: message_id={}, reason={:?}",
                    msg.message_id, msg.reason
                );
                return Err(anyhow::anyhow!("TCS sent close message: {:?}", msg.reason));
            }
        }

        Ok(())
    }
}
