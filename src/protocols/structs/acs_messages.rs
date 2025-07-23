use super::common::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum ACSMessage {
    HeartbeatMessage(HeartbeatMessageStruct),
    HeartbeatAckRequest(HeartbeatAckRequestStruct),
    TaskManifestMessage(TaskManifestMessageStruct),
    TaskStopVerificationMessage(TaskStopVerificationMessageStruct),
    TaskStopVerificationAck(TaskStopVerificationAckStruct),
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
    pub task_arn: String,
    // Minimal stub - only fields needed for ACK functionality
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmAttachmentMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub task_arn: String,
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
pub struct AckRequestStruct {
    pub message_id: String,
    pub cluster: Option<String>,
    pub container_instance: Option<String>,
    pub generated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IAMRoleCredentialsMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub task_arn: String,
    pub role_credentials: IAMRoleCredentials,
    pub role_type: Option<String>,
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
pub struct RefreshCredentialsMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub task_arn: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshCredentialsAckRequestStruct {
    pub message_id: String,
    pub generated_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IAMRoleCredentials {
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub expiration: Option<String>,
    pub role_arn: Option<String>,
    pub credentials_id: Option<String>,
}
