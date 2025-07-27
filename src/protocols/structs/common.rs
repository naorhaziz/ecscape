use serde::{Deserialize, Serialize};

// Common structs used by both ACS and TCS
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
