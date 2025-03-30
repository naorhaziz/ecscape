use anyhow::{Result, anyhow};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

pub struct WSClient {
    request: Request<()>,
}

impl WSClient {
    pub fn new(request: Request<()>) -> Self {
        Self { request }
    }

    pub async fn connect(
        &self,
    ) -> Result<(
        Pin<Box<dyn Stream<Item = Result<WsMessage>> + Send>>,
        Arc<Mutex<Pin<Box<dyn Sink<WsMessage, Error = anyhow::Error> + Send>>>>,
    )> {
        let (ws_stream, response) = connect_async(self.request.clone()).await?;
        if response.status() != 101 {
            return Err(anyhow!(
                "Failed to establish WebSocket connection, status: {}",
                response.status()
            ));
        }
        let (ws_sink, mut ws_stream) = ws_stream.split();

        let ws_sink = Arc::new(Mutex::new(Box::pin(
            ws_sink.sink_map_err(|e| anyhow!("Send error: {:?}", e)),
        )
            as Pin<Box<dyn Sink<WsMessage, Error = anyhow::Error> + Send>>));

        let ws_sink_clone = ws_sink.clone();
        let recv_stream = Box::pin(async_stream::stream! {
            while let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(WsMessage::Text(_)) | Ok(WsMessage::Binary(_)) => {
                        yield Ok(msg?);
                    }
                    Ok(WsMessage::Ping(ping_data)) => {
                        let mut sink = ws_sink_clone.lock().await;
                        sink.send(WsMessage::Pong(ping_data)).await?;
                    }
                    Ok(WsMessage::Pong(_)) => {}
                    Ok(WsMessage::Frame(_)) => {}
                    Ok(WsMessage::Close(close_data)) => {
                        yield Err(anyhow!("Connection closed: {:?}", close_data));
                        break;
                    }
                    Err(err) => {
                        yield Err(anyhow!("WebSocket error: {:?}", err));
                        break;
                    }
                }
            }
        });

        Ok((recv_stream, ws_sink))
    }
}
