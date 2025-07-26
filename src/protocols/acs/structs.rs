use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum ACSMessage {
    HeartbeatMessage(HeartbeatMessageStruct),
    HeartbeatAckRequest(HeartbeatAckRequestStruct),
    TaskManifestMessage(TaskManifestMessageStruct),
    TaskStopVerificationMessage(TaskStopVerificationMessageStruct),
    TaskStopVerificationAck(TaskStopVerificationAckStruct),
    PublishMetricsRequest(PublishMetricsRequestStruct),
    PublishInstanceStatusRequest(PublishInstanceStatusRequestStruct),
    IAMRoleCredentialsMessage(IAMRoleCredentialsMessageStruct),
    IAMRoleCredentialsAckRequest(IAMRoleCredentialsAckRequestStruct),
    RefreshCredentialsMessage(RefreshCredentialsMessageStruct),
    RefreshCredentialsAckRequest(RefreshCredentialsAckRequestStruct),
    PayloadMessage(PayloadMessageStruct),
    AttachTaskNetworkInterfacesMessage(AttachTaskNetworkInterfacesMessageStruct),
    AttachInstanceNetworkInterfacesMessage(AttachInstanceNetworkInterfacesMessageStruct),
    ConfirmAttachmentMessage(ConfirmAttachmentMessageStruct),
    AckRequest(AckRequestStruct),
    ErrorMessage(ErrorMessageStruct),
    CloseMessage(CloseMessageStruct),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatMessageStruct {
    pub message_id: String,
    pub healthy: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatAckRequestStruct {
    pub message_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskManifestMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub tasks: Option<Vec<TaskIdentifier>>,
    pub timeline: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskStopVerificationMessageStruct {
    pub message_id: String,
    pub stop_candidates: Option<Vec<TaskIdentifier>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskStopVerificationAckStruct {
    pub message_id: String,
    pub generated_at: Option<i64>,
    pub stop_tasks: Option<Vec<TaskIdentifier>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskIdentifier {
    pub task_arn: Option<String>,
    pub task_cluster_arn: Option<String>,
    pub desired_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    // Essential fields for credential capture and ACK responses
    pub arn: Option<String>, // Used in ACK logging
    pub execution_role_credentials: Option<IAMRoleCredentials>, // Primary credential field
    pub role_credentials: Option<IAMRoleCredentials>, // Task role credentials
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttachTaskNetworkInterfacesMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub task_arn: String,
    // Minimal stub - only fields needed for ACK functionality
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttachInstanceNetworkInterfacesMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    // Minimal stub - only fields needed for ACK functionality
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AckRequestStruct {
    pub message_id: String,
    pub cluster: String,
    pub container_instance: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmAttachmentMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub task_arn: Option<String>,
    pub task_cluster_arn: Option<String>,
    // Minimal stub - only fields needed for ACK functionality
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub tasks: Option<Vec<Task>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishMetricsRequestStruct {
    pub message_id: String,
    // Minimal stub - not used for credential capture
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishInstanceStatusRequestStruct {
    pub message_id: String,
    // Minimal stub - not used for credential capture
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IAMRoleCredentialsMessageStruct {
    pub message_id: String,
    pub task_arn: Option<String>,
    pub role_type: Option<String>,
    pub role_credentials: Option<IAMRoleCredentials>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshCredentialsMessageStruct {
    pub message_id: String,
    pub task_arn: Option<String>,
    pub role_type: Option<String>,
    pub role_credentials: Option<IAMRoleCredentials>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshCredentialsAckRequestStruct {
    pub message_id: String,
    pub task_arn: Option<String>,
    pub expiration: Option<String>,
    pub credentials_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IAMRoleCredentials {
    pub credentials_id: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub role_arn: Option<String>,
    pub expiration: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IAMRoleCredentialsAckRequestStruct {
    pub message_id: String,
    pub expiration: Option<String>,
    pub credentials_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessageStruct {
    pub message_id: String,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloseMessageStruct {
    pub message_id: String,
    pub reason: Option<String>,
}
