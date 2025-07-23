use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum TCSMessage {
    // TCS inbound messages (what TCS sends to us)
    StopTelemetrySessionMessage(StopTelemetrySessionMessageStruct),
    AckPublishMetric(AckPublishMetricStruct),
    HeartbeatMessage(HeartbeatMessageStruct),
    AckPublishHealth(AckPublishHealthStruct),
    AckPublishInstanceStatus(AckPublishInstanceStatusStruct),

    // TCS outbound messages (what we send to TCS)
    PublishMetricsRequest(PublishMetricsRequestStruct),
    PublishHealthRequest(PublishHealthRequestStruct),
    StartTelemetrySessionRequest(StartTelemetrySessionRequestStruct),
    PublishInstanceStatusRequest(PublishInstanceStatusRequestStruct),

    // TCS exception messages
    ServerException(ServerExceptionStruct),
    BadRequestException(BadRequestExceptionStruct),
    ResourceValidationException(ResourceValidationExceptionStruct),
    InvalidParameterException(InvalidParameterExceptionStruct),
    ErrorMessage(ErrorMessageStruct),
}

// TCS heartbeat message (inbound from TCS)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatMessageStruct {
    pub healthy: Option<bool>,
}

// TCS outbound message structs (what we send to TCS) - Based on Go API
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishMetricsRequestStruct {
    pub instance_metrics: Option<InstanceMetrics>,
    pub metadata: Option<MetricsMetadata>,
    pub task_metrics: Option<Vec<TaskMetric>>,
    pub timestamp: Option<i64>, // utils.Timestamp in Go
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishHealthRequestStruct {
    pub metadata: Option<HealthMetadata>,
    pub tasks: Option<Vec<TaskHealth>>,
    pub timestamp: Option<i64>, // utils.Timestamp in Go
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishInstanceStatusRequestStruct {
    pub metadata: Option<InstanceStatusMetadata>,
    pub statuses: Option<Vec<InstanceStatus>>,
    pub timestamp: Option<i64>, // utils.Timestamp in Go
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartTelemetrySessionRequestStruct {
    pub cluster: Option<String>,
    pub container_instance: Option<String>,
}

// Supporting metadata structures - Based on Go API
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MetricsMetadata {
    pub cluster: Option<String>,
    pub container_instance: Option<String>,
    pub fin: Option<bool>,
    pub idle: Option<bool>,
    pub message_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HealthMetadata {
    pub cluster: Option<String>,
    pub container_instance: Option<String>,
    pub fin: Option<bool>,
    pub message_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatusMetadata {
    pub cluster: Option<String>,
    pub container_instance: Option<String>,
    pub request_id: Option<String>,
}

// Instance metrics structure - Based on Go API
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceMetrics {
    pub storage: Option<InstanceStorageMetrics>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStorageMetrics {
    pub data_filesystem: Option<f64>,
    pub root_filesystem: Option<f64>,
}

// Task-related structures - Based on Go API
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskMetric {
    pub cluster_arn: Option<String>,
    pub container_metrics: Option<Vec<ContainerMetric>>,
    pub ephemeral_storage_metrics: Option<EphemeralStorageMetrics>,
    pub service_connect_metrics_wrapper: Option<Vec<GeneralMetricsWrapper>>,
    pub task_arn: Option<String>,
    pub task_definition_family: Option<String>,
    pub task_definition_version: Option<String>,
    pub volume_metrics: Option<Vec<VolumeMetric>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskHealth {
    pub cluster_arn: Option<String>,
    pub containers: Option<Vec<ContainerHealth>>,
    pub task_arn: Option<String>,
    pub task_definition_family: Option<String>,
    pub task_definition_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatus {
    pub last_status_change: Option<i64>, // utils.Timestamp in Go
    pub last_updated: Option<i64>,       // utils.Timestamp in Go
    pub status: Option<String>,          // enum:"InstanceHealthcheckStatus"
    pub r#type: Option<String>,          // "type" is reserved keyword, use r#type
}

// Container-related structures
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContainerHealth {
    pub container_name: Option<String>,
    pub health_status: Option<String>, // enum:"HealthStatus"
    pub status_message: Option<String>,
}

// Placeholder structures for complex nested types - Based on Go API
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContainerMetric {
    pub container_arn: Option<String>,
    pub container_name: Option<String>,
    pub cpu_stats_set: Option<CWStatsSet>,
    pub memory_stats_set: Option<CWStatsSet>,
    pub network_stats_set: Option<NetworkStatsSet>,
    pub restart_stats_set: Option<RestartStatsSet>,
    pub storage_stats_set: Option<StorageStatsSet>,
}

// Supporting metrics structures - Based on Go API
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CWStatsSet {
    pub max: Option<f64>,
    pub min: Option<f64>,
    pub sample_count: Option<i64>,
    pub sum: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatsSet {
    pub rx_bytes: Option<ULongStatsSet>,
    pub rx_bytes_per_second: Option<UDoubleCWStatsSet>,
    pub rx_dropped: Option<ULongStatsSet>,
    pub rx_errors: Option<ULongStatsSet>,
    pub rx_packets: Option<ULongStatsSet>,
    pub tx_bytes: Option<ULongStatsSet>,
    pub tx_bytes_per_second: Option<UDoubleCWStatsSet>,
    pub tx_dropped: Option<ULongStatsSet>,
    pub tx_errors: Option<ULongStatsSet>,
    pub tx_packets: Option<ULongStatsSet>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestartStatsSet {
    pub exit_code: Option<i64>,
    pub last_exit_time: Option<i64>,  // utils.Timestamp in Go
    pub last_start_time: Option<i64>, // utils.Timestamp in Go
    pub restart_count: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StorageStatsSet {
    pub name: Option<String>,
    pub read_bytes_per_second: Option<UDoubleCWStatsSet>,
    pub read_ios_per_second: Option<UDoubleCWStatsSet>,
    pub total_bytes_per_second: Option<UDoubleCWStatsSet>,
    pub total_ios_per_second: Option<UDoubleCWStatsSet>,
    pub write_bytes_per_second: Option<UDoubleCWStatsSet>,
    pub write_ios_per_second: Option<UDoubleCWStatsSet>,
}

// Placeholder for complex stat types - simplified for now
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ULongStatsSet {
    pub max: Option<u64>,
    pub min: Option<u64>,
    pub sample_count: Option<i64>,
    pub sum: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UDoubleCWStatsSet {
    pub max: Option<f64>,
    pub min: Option<f64>,
    pub sample_count: Option<i64>,
    pub sum: Option<f64>,
}

// Remaining placeholder structures (to be expanded as needed)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EphemeralStorageMetrics {
    pub utilized: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GeneralMetricsWrapper {
    // Service Connect metrics wrapper - add fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VolumeMetric {
    pub name: Option<String>,
    pub utilized_bytes: Option<i64>,
}

// TCS inbound message structs (what TCS sends to us)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AckPublishMetricStruct {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AckPublishHealthStruct {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AckPublishInstanceStatusStruct {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopTelemetrySessionMessageStruct {
    pub message: Option<String>,
}

// TCS exception message structs (AWS SDK pattern)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerExceptionStruct {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BadRequestExceptionStruct {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceValidationExceptionStruct {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InvalidParameterExceptionStruct {
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessageStruct {
    pub message: Option<String>,
}
