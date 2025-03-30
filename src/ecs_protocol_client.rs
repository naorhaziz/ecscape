use crate::structs::{HeartbeatAckRequest, ProtocolMessage};
use crate::ws_client::WSClient;
use anyhow::{Result, anyhow};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

pub struct ECSProtocolClient {
    ws_client: WSClient,
}

impl ECSProtocolClient {
    pub fn new(request: Request<()>) -> Self {
        Self {
            ws_client: WSClient::new(request),
        }
    }

    pub async fn connect(
        &self,
    ) -> Result<(
        Pin<Box<dyn Stream<Item = Result<ProtocolMessage>> + Send>>,
        Arc<Mutex<Pin<Box<dyn Sink<ProtocolMessage, Error = anyhow::Error> + Send>>>>,
    )> {
        let (mut recv_stream, send_sink) = self.ws_client.connect().await?;

        let protocol_send_sink: Arc<
            Mutex<Pin<Box<dyn Sink<ProtocolMessage, Error = anyhow::Error> + Send>>>,
        > = Arc::new(Mutex::new(Box::pin(futures_util::sink::unfold(
            send_sink,
            |sink, msg: ProtocolMessage| async move {
                let json_msg = serde_json::to_string(&msg)?;
                sink.lock()
                    .await
                    .send(WsMessage::Text(json_msg.into()))
                    .await?;
                Ok(sink)
            },
        ))));

        let protocol_send_sink_clone = protocol_send_sink.clone();
        let protocol_recv_stream = Box::pin(async_stream::stream! {
            while let Some(msg) = recv_stream.next().await {
                match msg {
                    Ok(WsMessage::Text(text)) => {
                        let protocol_msg = serde_json::from_str::<ProtocolMessage>(&text)?;
                        match protocol_msg {
                            ProtocolMessage::HeartbeatMessage(heartbeat) => {
                                let ack = ProtocolMessage::HeartbeatAckRequest(HeartbeatAckRequest {
                                    message_id: heartbeat.message_id,
                                });
                                println!("Received heartbeat, sending ack: {:#?}", ack);
                                protocol_send_sink_clone.lock().await.send(ack).await?;
                            },
                            ProtocolMessage::ErrorMessage(error) => {
                                yield Err(anyhow!("Received error message: {:?}", error));
                                break;
                            },
                            ProtocolMessage::CloseMessage(close) => {
                                yield Err(anyhow!("Received close message: {:?}", close));
                                break;
                            }
                            _ => {
                                yield Ok(protocol_msg);
                            },
                        }
                    },
                    Err(err) => yield Err(anyhow!("WebSocket error: {:?}", err)),
                    _ => { yield Err(anyhow!("Unsupported WebSocket message"))},
                }
            }
        });

        Ok((protocol_recv_stream, protocol_send_sink))
    }
}
