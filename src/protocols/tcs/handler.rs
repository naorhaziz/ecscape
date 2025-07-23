use anyhow::Result;
use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sdk_ecs::operation::discover_poll_endpoint::DiscoverPollEndpointOutput;
use tokio::{select, time};
use tokio_tungstenite::tungstenite::http::Request;
use tracing::{debug, error, info, warn};

use crate::protocols::{
    protocol_client::ProtocolClient,
    protocol_handler::ProtocolHandler,
    request_builder::RequestBuilder,
    tcs::{
        request_builder::TCSRequestBuilder,
        structs::{
            HealthMetadata, InstanceMetrics, InstanceStatusMetadata, InstanceStorageMetrics,
            MetricsMetadata, PublishHealthRequestStruct, PublishInstanceStatusRequestStruct,
            PublishMetricsRequestStruct, TCSMessage,
        },
    },
};

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

    async fn handle_message(
        &self,
        _client: &mut ProtocolClient<TCSMessage>,
        message: TCSMessage,
    ) -> Result<()> {
        match message {
            TCSMessage::HeartbeatMessage(msg) => {
                info!("Processing TCS HeartbeatMessage: {:?}", msg);
                // TCS HeartbeatMessage doesn't have a message_id and doesn't require acknowledgment
                // We just log that we received it - this indicates the connection is healthy
                debug!("TCS heartbeat received, connection is healthy");
            }

            TCSMessage::AckPublishMetric(msg) => {
                info!("Received AckPublishMetric from TCS: {:?}", msg);
            }

            TCSMessage::AckPublishHealth(msg) => {
                info!("Received AckPublishHealth from TCS: {:?}", msg);
            }

            TCSMessage::AckPublishInstanceStatus(msg) => {
                info!("Received AckPublishInstanceStatus from TCS: {:?}", msg);
            }

            TCSMessage::StopTelemetrySessionMessage(msg) => {
                warn!(
                    "Received StopTelemetrySessionMessage from TCS: message={:?}",
                    msg
                );
                return Err(anyhow::anyhow!(
                    "TCS requested to stop telemetry session: {:?}",
                    msg.message
                ));
            }

            // These are messages we send, not messages we should respond to
            TCSMessage::PublishMetricsRequest(_)
            | TCSMessage::PublishHealthRequest(_)
            | TCSMessage::PublishInstanceStatusRequest(_)
            | TCSMessage::StartTelemetrySessionRequest(_) => {
                debug!("Received outbound message - no action needed");
            }

            TCSMessage::ServerException(msg) => {
                warn!(
                    "Received ServerException from TCS: message={:?}",
                    msg.message
                );
                return Err(anyhow::anyhow!("TCS server exception: {:?}", msg.message));
            }

            TCSMessage::BadRequestException(msg) => {
                warn!(
                    "Received BadRequestException from TCS: message={:?}",
                    msg.message
                );
                return Err(anyhow::anyhow!("TCS bad request: {:?}", msg.message));
            }

            TCSMessage::ResourceValidationException(msg) => {
                warn!(
                    "Received ResourceValidationException from TCS: message={:?}",
                    msg.message
                );
                return Err(anyhow::anyhow!(
                    "TCS resource validation error: {:?}",
                    msg.message
                ));
            }

            TCSMessage::InvalidParameterException(msg) => {
                warn!(
                    "Received InvalidParameterException from TCS: message={:?}",
                    msg.message
                );
                return Err(anyhow::anyhow!("TCS invalid parameter: {:?}", msg.message));
            }

            TCSMessage::ErrorMessage(msg) => {
                warn!("Received ErrorMessage from TCS: {:?}", msg);
                return Err(anyhow::anyhow!("TCS error: {:?}", msg.message));
            }
        }

        Ok(())
    }

    async fn publish_metrics(&self, client: &mut ProtocolClient<TCSMessage>) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Create basic instance storage metrics
        let instance_storage_metrics = InstanceStorageMetrics {
            data_filesystem: Some(0.0), // Placeholder - should be real disk usage
            root_filesystem: Some(0.0), // Placeholder - should be real disk usage
        };

        let instance_metrics = InstanceMetrics {
            storage: Some(instance_storage_metrics),
        };

        // Detect if instance is idle (no tasks running)
        let is_idle = true; // TODO: Implement actual idle detection based on running tasks

        let metadata = MetricsMetadata {
            cluster: Some(self.cluster_arn.clone()),
            container_instance: Some(self.container_instance_arn.clone()),
            fin: Some(is_idle), // Set fin=true for idle instances (Go agent pattern)
            idle: Some(is_idle),
            message_id: Some(uuid::Uuid::new_v4().to_string()),
        };

        let publish_request = if is_idle {
            // For idle instances, Go agent sends minimal request with no task metrics
            // and no instance metrics (following filterInstanceMetrics logic)
            PublishMetricsRequestStruct {
                instance_metrics: None,
                metadata: Some(metadata),
                task_metrics: None,
                timestamp: Some(timestamp),
            }
        } else {
            // For active instances, include instance metrics
            PublishMetricsRequestStruct {
                instance_metrics: Some(instance_metrics),
                metadata: Some(metadata),
                task_metrics: None, // TODO: Collect actual task metrics
                timestamp: Some(timestamp),
            }
        };

        let message = TCSMessage::PublishMetricsRequest(publish_request);
        client.send(&message).await?;

        debug!("Published metrics to TCS (idle: {})", is_idle);
        Ok(())
    }

    async fn publish_health(&self, client: &mut ProtocolClient<TCSMessage>) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let metadata = HealthMetadata {
            cluster: Some(self.cluster_arn.clone()),
            container_instance: Some(self.container_instance_arn.clone()),
            fin: Some(true), // Always true for health messages (Go agent pattern)
            message_id: Some(uuid::Uuid::new_v4().to_string()),
        };

        let publish_request = PublishHealthRequestStruct {
            metadata: Some(metadata),
            tasks: None, // Placeholder - should collect actual task health
            timestamp: Some(timestamp),
        };

        let message = TCSMessage::PublishHealthRequest(publish_request);
        client.send(&message).await?;

        debug!("Published health to TCS");
        Ok(())
    }

    async fn publish_instance_status(&self, client: &mut ProtocolClient<TCSMessage>) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let metadata = InstanceStatusMetadata {
            cluster: Some(self.cluster_arn.clone()),
            container_instance: Some(self.container_instance_arn.clone()),
            request_id: Some(uuid::Uuid::new_v4().to_string()),
        };

        let publish_request = PublishInstanceStatusRequestStruct {
            metadata: Some(metadata),
            statuses: None, // Placeholder - should collect actual instance status
            timestamp: Some(timestamp),
        };

        let message = TCSMessage::PublishInstanceStatusRequest(publish_request);
        client.send(&message).await?;

        debug!("Published instance status to TCS");
        Ok(())
    }
}

#[async_trait]
impl ProtocolHandler<TCSMessage> for TCSHandler {
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

    async fn start_inner(&self, mut client: ProtocolClient<TCSMessage>) -> Result<()> {
        // Constants matching the real ECS agent intervals
        const METRICS_PUBLISH_INTERVAL: time::Duration = time::Duration::from_secs(20); // DefaultContainerMetricsPublishInterval = 20s
        const HEALTH_PUBLISH_INTERVAL: time::Duration = time::Duration::from_secs(20); // Health is published with same frequency as metrics
        const INSTANCE_STATUS_PUBLISH_INTERVAL: time::Duration = time::Duration::from_secs(60); // Instance status less frequent

        info!("TCS connection established, starting periodic telemetry publishing");

        // Create intervals for different types of telemetry
        // Note: The Go agent uses channels and only publishes when there's actual data
        // For now, we'll use timers but with longer intervals when idle
        let mut metrics_interval = time::interval(METRICS_PUBLISH_INTERVAL);
        let mut health_interval = time::interval(HEALTH_PUBLISH_INTERVAL);
        let mut status_interval = time::interval(INSTANCE_STATUS_PUBLISH_INTERVAL);

        // Skip the first tick to avoid immediate publishing
        metrics_interval.tick().await;
        health_interval.tick().await;
        status_interval.tick().await;

        loop {
            select! {
                // Handle incoming messages
                incoming = client.receive() => {
                    match incoming {
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

                // Publish metrics periodically
                _ = metrics_interval.tick() => {
                    if let Err(err) = self.publish_metrics(&mut client).await {
                        warn!("Failed to publish metrics: {:?}", err);
                        return Err(err);
                    }
                }

                // Publish health periodically
                _ = health_interval.tick() => {
                    if let Err(err) = self.publish_health(&mut client).await {
                        warn!("Failed to publish health: {:?}", err);
                        return Err(err);
                    }
                }

                // Publish instance status periodically
                _ = status_interval.tick() => {
                    if let Err(err) = self.publish_instance_status(&mut client).await {
                        warn!("Failed to publish instance status: {:?}", err);
                        return Err(err);
                    }
                }
            }
        }
    }
}
