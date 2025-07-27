use serde::{Deserialize, Serialize};
use super::common::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum TCSMessage {
    // Common messages (heartbeat)
    HeartbeatMessage(HeartbeatMessageStruct),
    HeartbeatAckRequest(HeartbeatAckRequestStruct),
    
    // TCS-specific outbound messages (what we send to TCS)
    PublishMetricsRequest(PublishMetricsRequestStruct),
    PublishHealthRequest(PublishHealthRequestStruct),
    PublishInstanceStatusRequest(PublishInstanceStatusRequestStruct),
    StartTelemetrySessionRequest(StartTelemetrySessionRequestStruct),
    
    // TCS-specific inbound messages (what TCS sends to us)
    AckPublishMetric(AckPublishMetricStruct),
    AckPublishHealth(AckPublishHealthStruct),
    AckPublishInstanceStatus(AckPublishInstanceStatusStruct),
    StopTelemetrySessionMessage(StopTelemetrySessionMessageStruct),
    
    // Common error/close messages
    ErrorMessage(ErrorMessageStruct),
    CloseMessage(CloseMessageStruct),
}

// TCS outbound message structs (what we send)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishMetricsRequestStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub timestamp: Option<i64>,
    pub metrics: Vec<MetricData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishHealthRequestStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub timestamp: Option<i64>,
    pub health_metrics: Vec<HealthMetric>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishInstanceStatusRequestStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub timestamp: Option<i64>,
    pub status: String,
    pub generated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartTelemetrySessionRequestStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub container_instance_tags: Option<std::collections::HashMap<String, String>>,
}

// TCS inbound message structs (what TCS sends to us)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AckPublishMetricStruct {
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]  
pub struct AckPublishHealthStruct {
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AckPublishInstanceStatusStruct {
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopTelemetrySessionMessageStruct {
    pub message_id: String,
    pub reason: Option<String>,
}

// Supporting data structures for TCS messages
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MetricData {
    pub metric_name: String,
    pub value: f64,
    pub unit: Option<String>,
    pub timestamp: Option<i64>,
    pub dimensions: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HealthMetric {
    pub metric_name: String,
    pub status: String, // "OK", "WARNING", "CRITICAL"
    pub timestamp: Option<i64>,
    pub details: Option<String>,
}
