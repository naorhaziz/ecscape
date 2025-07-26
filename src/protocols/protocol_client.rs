use crate::ws_client::WSClient;
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use tokio_tungstenite::tungstenite::http::Request;

pub struct ProtocolClient<T> {
    ws_client: WSClient,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ProtocolClient<T>
where
    T: Serialize + DeserializeOwned,
{
    pub async fn connect(request: Request<()>) -> Result<Self> {
        let ws_client = WSClient::connect(request).await?;
        Ok(Self {
            ws_client,
            _phantom: std::marker::PhantomData,
        })
    }

    pub async fn send(&self, message: &T) -> Result<()> {
        self.ws_client.send(serde_json::to_string(message)?).await
    }

    pub async fn receive(&mut self) -> Result<Option<T>> {
        match self.ws_client.receive().await? {
            Some(msg) => Ok(Some(serde_json::from_str::<T>(&msg)?)),
            None => Ok(None),
        }
    }
}
