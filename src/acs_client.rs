use anyhow::Result;
use serde_json;

use crate::{acs_connector::ACSConnector, structs::ProtocolMessage, ws_client::WSClient};

pub struct ACSClient {
    ws_client: WSClient,
}

impl ACSClient {
    pub async fn connect() -> Result<Self> {
        let acs_connector = ACSConnector::try_new().await?;
        let ws_client = acs_connector.connect().await?;

        Ok(Self { ws_client })
    }

    pub async fn send(&mut self, message: &ProtocolMessage) -> Result<()> {
        self.ws_client.send(serde_json::to_string(message)?).await
    }

    pub async fn receive(&mut self) -> Result<Option<ProtocolMessage>> {
        match self.ws_client.receive().await? {
            Some(msg) => Ok(Some(serde_json::from_str::<ProtocolMessage>(&msg)?)),
            None => Ok(None),
        }
    }
}
