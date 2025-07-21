use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "message")]
pub enum ProtocolMessage {
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
    pub arn: Option<String>,
    pub definition_key: Option<String>,
    pub family: Option<String>,
    pub version: Option<String>,
    pub desired_status: Option<String>,
    pub known_status: Option<String>,
    pub containers: Option<Vec<Container>>,
    pub volumes: Option<Vec<Volume>>,
    pub role_credentials: Option<IAMRoleCredentials>,
    pub task_role_arn: Option<String>,
    pub execution_role_arn: Option<String>,
    pub platform_version: Option<String>,
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub elastic_network_interfaces: Option<Vec<ElasticNetworkInterface>>,
    pub attachments: Option<Vec<Attachment>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttachTaskNetworkInterfacesMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub task_arn: String,
    pub elastic_network_interfaces: Vec<ElasticNetworkInterface>,
    pub wait_timeout_ms: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttachInstanceNetworkInterfacesMessageStruct {
    pub message_id: String,
    pub cluster_arn: String,
    pub container_instance_arn: String,
    pub elastic_network_interfaces: Vec<ElasticNetworkInterface>,
    pub wait_timeout_ms: i64,
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
pub struct TaskManifestAckRequest {
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
    pub attachment: Attachment,
    pub wait_timeout_ms: Option<i64>,
}

// ENI and Network Interface Structs
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElasticNetworkInterface {
    pub attachment_arn: Option<String>,
    pub ec2_id: String,      // Required - validated as non-empty
    pub mac_address: String, // Required - validated as non-empty
    pub ipv4_addresses: Vec<IPv4AddressAssignment>, // Required - validated as non-empty
    pub ipv6_addresses: Option<Vec<IPv6AddressAssignment>>,
    pub subnet_gateway_ipv4_address: String, // Required - validated as non-empty
    pub subnet_gateway_ipv6_address: Option<String>,
    pub domain_name_servers: Option<Vec<String>>,
    pub domain_name: Option<Vec<String>>,
    pub private_dns_name: Option<String>,
    pub interface_association_protocol: Option<String>,
    pub interface_vlan_properties: Option<InterfaceVlanProperties>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IPv4AddressAssignment {
    pub primary: Option<bool>,
    pub private_address: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IPv6AddressAssignment {
    pub address: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceVlanProperties {
    pub vlan_id: Option<String>,
    pub trunk_interface_mac_address: Option<String>,
}

// Attachment Structs
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub attachment_arn: Option<String>,
    pub attachment_type: Option<String>,
    pub attachment_properties: Option<Vec<AttachmentProperty>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentProperty {
    pub name: Option<String>,
    pub value: Option<String>,
}

// Extended Container struct with more fields
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub docker_id: Option<String>,
    pub name: Option<String>,
    pub docker_name: Option<String>,
    pub image: Option<String>,
    pub image_id: Option<String>,
    pub ports: Option<Vec<PortBinding>>,
    pub volumes: Option<Vec<VolumeMount>>,
    pub environment: Option<HashMap<String, String>>,
    pub desired_status: Option<String>,
    pub known_status: Option<String>,
    pub exit_code: Option<i32>,
    pub reason: Option<String>,
    pub created_at: Option<i64>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub networks: Option<Vec<Network>>,
    pub links: Option<Vec<String>>,
    pub essential: Option<bool>,
    pub cpu: Option<i32>,
    pub memory: Option<i32>,
    pub memory_reservation: Option<i32>,
    pub port_mappings: Option<Vec<PortMapping>>,
    pub mount_points: Option<Vec<MountPoint>>,
    pub volumes_from: Option<Vec<VolumeFrom>>,
    pub log_configuration: Option<LogConfiguration>,
    pub health_check_grace_period_seconds: Option<i32>,
    pub network_interface_names: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PortMapping {
    pub container_port: Option<i32>,
    pub host_port: Option<i32>,
    pub protocol: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MountPoint {
    pub source_volume: Option<String>,
    pub container_path: Option<String>,
    pub read_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VolumeFrom {
    pub source_container: Option<String>,
    pub read_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogConfiguration {
    pub log_driver: Option<String>,
    pub options: Option<HashMap<String, String>>,
}

// Extended Volume struct with all configurations
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub volume_type: Option<String>,
    pub source: Option<String>,
    pub scope: Option<String>,
    pub autoprovision: Option<bool>,
    pub driver: Option<String>,
    pub driver_opts: Option<HashMap<String, String>>,
    pub labels: Option<HashMap<String, String>>,
    pub host: Option<HostVolumeProperties>,
    pub efs_volume_configuration: Option<EFSVolumeConfiguration>,
    pub ebs_volume_configuration: Option<EBSVolumeConfiguration>,
    pub fsx_windows_file_server_volume_configuration:
        Option<FSxWindowsFileServerVolumeConfiguration>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HostVolumeProperties {
    pub source_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EFSVolumeConfiguration {
    pub file_system_id: Option<String>,
    pub root_directory: Option<String>,
    pub transit_encryption: Option<String>,
    pub transit_encryption_port: Option<i32>,
    pub authorization_config: Option<EFSAuthorizationConfig>,
    pub docker_volume_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EFSAuthorizationConfig {
    pub access_point_id: Option<String>,
    pub iam: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EBSVolumeConfiguration {
    pub volume_id: Option<String>,
    pub volume_name: Option<String>,
    pub volume_size_gib: Option<String>,
    pub source_volume_host_path: Option<String>,
    pub device_name: Option<String>,
    pub file_system: Option<String>,
    pub docker_volume_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FSxWindowsFileServerVolumeConfiguration {
    pub file_system_id: Option<String>,
    pub root_directory: Option<String>,
    pub authorization_config: Option<FSxWindowsFileServerAuthorizationConfig>,
    pub docker_volume_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FSxWindowsFileServerAuthorizationConfig {
    pub credentials_parameter: Option<String>,
    pub domain: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadMessageStruct {
    pub message_id: String,             // Required - validated as non-empty
    pub cluster_arn: String,            // Required - validated as non-empty
    pub container_instance_arn: String, // Required - validated as non-empty
    pub tasks: Option<Vec<Task>>,
    pub generated_at: Option<i64>,
    pub seq_num: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishMetricsRequestStruct {
    pub message_id: String,
    pub timestamp: Option<i64>,
    pub task_arn: Option<String>,
    pub metrics: Option<Vec<TaskMetric>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskMetric {
    pub container_name: Option<String>,
    pub cpu_utilization: Option<f64>,
    pub memory_utilization: Option<f64>,
    pub memory_reservation: Option<u64>,
    pub network_rx_bytes: Option<u64>,
    pub network_tx_bytes: Option<u64>,
    pub storage_read_bytes: Option<u64>,
    pub storage_write_bytes: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublishInstanceStatusRequestStruct {
    pub message_id: String,
    pub container_instance_arn: Option<String>,
    pub status: Option<String>,
    pub remaining_resources: Option<Vec<Resource>>,
    pub registered_resources: Option<Vec<Resource>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub double_value: Option<f64>,
    pub long_value: Option<i64>,
    pub integer_value: Option<i32>,
    pub string_set_value: Option<Vec<String>>,
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
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
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
pub struct PortBinding {
    pub host_port: Option<u16>,
    pub container_port: Option<u16>,
    pub protocol: Option<String>,
    pub bind_ip: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VolumeMount {
    pub name: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub read_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub network_mode: Option<String>,
    pub ipv4_addresses: Option<Vec<String>>,
    pub ipv6_addresses: Option<Vec<String>>,
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
