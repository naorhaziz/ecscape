use crate::{protocols::structs::ProtocolMessage, ws_client::WSClient};
use anyhow::Result;
use serde_json;
use tokio_tungstenite::tungstenite::http::Request;

pub struct ECSProtocolClient {
    ws_client: WSClient,
}

impl ECSProtocolClient {
    pub async fn connect(request: Request<()>) -> Result<Self> {
        let ws_client = WSClient::connect(request).await?;
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
