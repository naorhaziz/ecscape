use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum ProtocolMessage {
    HeartbeatMessage(HeartbeatMessage),
    HeartbeatAckRequest(HeartbeatAckRequest),
    TaskManifestMessage(TaskManifestMessage),
    PublishMetricsRequest(PublishMetricsRequest),
    PublishInstanceStatusRequest(PublishInstanceStatusRequest),
    IAMRoleCredentialsMessage(IAMRoleCredentialsMessage),
    IAMRoleCredentialsAckRequest(IAMRoleCredentialsAckRequest),
    PayloadMessage(PayloadMessage),
    ErrorMessage(ErrorMessage),
    CloseMessage(CloseMessage),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloseMessage {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatMessage {
    pub healthy: bool,
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatAckRequest {
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskIdentifier {
    pub task_arn: String,
    pub task_cluster_arn: String,
    pub desired_status: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskManifestMessage {
    pub container_instance_arn: String,
    pub cluster_arn: String,
    pub tasks: Vec<TaskIdentifier>,
    pub generated_at: u64,
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatusMetadata {
    pub cluster: String,
    pub container_instance: String,
    pub request_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum InstanceStatusType {
    ContainerRuntime,
    TaskRuntime,
    AgentHealth,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstanceStatusStatus {
    Ok,
    Impaired,
    InsufficientData,
    Initializing,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatus {
    #[serde(rename = "type")]
    pub _type: InstanceStatusType,
    pub status: InstanceStatusStatus,
    pub last_updated: f64,
    pub last_status_change: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishInstanceStatusRequest {
    pub metadata: InstanceStatusMetadata,
    pub statuses: Vec<InstanceStatus>,
    pub timestamp: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MetricsMetadata {
    pub cluster: String,
    pub container_instance: String,
    pub message_id: String,
    pub fin: bool,
    pub idle: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishMetricsRequest {
    pub metadata: MetricsMetadata,
    pub timestamp: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IAMRoleCredentials {
    pub credentials_id: String,
    pub role_arn: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub expiration: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum RoleType {
    TaskApplication,
    TaskExecution,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IAMRoleCredentialsMessage {
    pub task_arn: String,
    pub task_cluster_arn: String,
    pub role_credentials: IAMRoleCredentials,
    pub role_type: RoleType,
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IAMRoleCredentialsAckRequest {
    pub message_id: String,
    pub expiration: String,
    pub credentials_id: String,
}

// Core message that contains tasks with execution role credentials
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadMessage {
    pub tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub execution_role_credentials: Option<IAMRoleCredentials>,
}
