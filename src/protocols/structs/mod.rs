pub mod acs_messages;
pub mod common;
pub mod tcs_messages;

pub use acs_messages::*;
pub use common::*;
pub use tcs_messages::*;

use serde::{Deserialize, Serialize};

// Main protocol message enum that can be either ACS or TCS
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ProtocolMessage {
    ACS(ACSMessage),
    TCS(TCSMessage),
}
