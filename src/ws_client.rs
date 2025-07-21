use anyhow::{Result, anyhow};
use aws_credential_types::Credentials;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4::SigningParams,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::time::{Duration, SystemTime};
use tokio::{
    net::TcpStream,
    select,
    sync::mpsc::{
        Receiver, Sender, UnboundedReceiver, UnboundedSender, channel, unbounded_channel,
    },
    task::{JoinHandle, block_in_place},
    time::{MissedTickBehavior, interval},
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Message,
        http::{Method, Request},
    },
};
use tracing::{debug, warn};
use url::Url;

pub struct WSClient {
    write_tx: UnboundedSender<String>,
    close_tx: Sender<()>,
    join_writer: JoinHandle<Result<()>>,
    reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WSClient {
    pub async fn connect_with_sigv4(
        url: Url,
        aws_region: &str,
        credentials: Credentials,
    ) -> Result<Self> {
        let request = Self::build_request(url, aws_region, credentials)?;

        let (ws_stream, response) = connect_async(request).await?;
        if response.status() != 101 {
            return Err(anyhow!(
                "Failed to establish WebSocket connection, status: {}",
                response.status()
            ));
        }

        let (sink, stream) = ws_stream.split();

        let (write_tx, write_rx) = unbounded_channel();
        let (close_tx, close_rx) = channel(1);
        let join_writer = tokio::spawn(Self::writer_task(close_rx, write_rx, sink));

        Ok(Self {
            close_tx,
            write_tx,
            join_writer,
            reader: stream,
        })
    }

    fn build_request(url: Url, aws_region: &str, credentials: Credentials) -> Result<Request<()>> {
        let signable_request = SignableRequest::new(
            "GET",
            url.as_str(),
            std::iter::empty(),
            SignableBody::Bytes(&[]),
        )?;

        let identity = credentials.into();

        let signing_params = SigningParams::builder()
            .identity(&identity)
            .region(aws_region)
            .name("ecs")
            .time(SystemTime::now())
            .settings(SigningSettings::default())
            .build()?
            .into();

        let (signing_instructions, _signature) =
            sign(signable_request, &signing_params)?.into_parts();

        let mut request = Request::builder()
            .method(Method::GET)
            .uri(url.as_str())
            .header("Host", url.host_str().ok_or(anyhow!("Missing host"))?)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .header("Sec-WebSocket-Version", "13")
            .body(())?;
        signing_instructions.apply_to_request_http1x(&mut request);

        Ok(request)
    }

    async fn writer_task(
        mut close_rx: Receiver<()>,
        mut write_rx: UnboundedReceiver<String>,
        mut sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ) -> Result<()> {
        const PING_INTERVAL: Duration = Duration::from_secs(30);

        let mut ping_ticker = interval(PING_INTERVAL);
        ping_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            select! {
                biased;
                _ = close_rx.recv() => {
                    debug!("WebSocket close channel closed, closing connection and stopping handler task");
                    sink.send(Message::Close(None)).await?;
                    break;
                }
                msg = write_rx.recv() => {
                    match msg {
                        Some(msg) => {
                            sink.send(Message::Text(msg.into())).await?;
                        }
                        None => {
                            debug!("WebSocket write channel closed, stopping writer task");
                            break;
                        }
                    }
                }
                _ = ping_ticker.tick() => {
                    sink.send(Message::Ping(Default::default())).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn send(&mut self, message: String) -> Result<()> {
        // Check if the writer task is still running
        if self.join_writer.is_finished() {
            return Err(anyhow!(
                "WebSocket writer task has finished, cannot send message"
            ));
        }

        self.write_tx.send(message)?;

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Option<String>> {
        loop {
            match self.reader.next().await {
                Some(Ok(msg)) => match msg {
                    Message::Text(text) => return Ok(Some(text.to_string())),
                    Message::Close(_) => {
                        debug!("WebSocket connection closed by server");
                        return Ok(None);
                    }
                    Message::Ping(_data) => {
                        // Tungsite automatically responds to pings with a pong
                    }
                    Message::Pong(_) => {}
                    _ => {
                        // ACS protocol shouldn't use binary messages
                        return Err(anyhow!("Unexpected WS message type"));
                    }
                },
                Some(Err(err)) => return Err(anyhow!("WebSocket error: {:?}", err)),
                None => return Ok(None),
            }
        }
    }
}

impl Drop for WSClient {
    fn drop(&mut self) {
        block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                if let Err(err) = self.close_tx.send(()).await {
                    warn!("Failed to send close signal: {:?}", err);
                }
            });
            let result = runtime.block_on(&mut self.join_writer);
            if let Err(err) = result {
                warn!("writer task did not stop properly: {:?}", err);
            }
        });
    }
}
