use anyhow::{Result, anyhow};
use aws_sdk_ecs::config::Credentials;
use aws_sigv4::http_request::{SignableBody, SignableRequest, SigningSettings, sign};
use aws_sigv4::sign::v4::SigningParams;
use std::time::SystemTime;
use tokio_tungstenite::tungstenite::http::Request;
use url::Url;

use crate::config::SEC_WEBSOCKET_VERSION;

pub fn build_ws_url(url: &str) -> Result<Url> {
    let mut url = Url::parse(url)?;

    match url.scheme() {
        "http" => url
            .set_scheme("ws")
            .map_err(|_| anyhow!("failed to set scheme to ws"))?,
        "https" => url
            .set_scheme("wss")
            .map_err(|_| anyhow!("failed to set scheme to wss"))?,
        scheme => return Err(anyhow!("unsupported scheme: {scheme}")),
    };

    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/ws", url.path()));
    } else {
        url.set_path(&format!("{}ws", url.path()));
    }

    Ok(url)
}

pub fn create_sigv4_signed_request(
    url: Url,
    aws_region: &str,
    aws_access_key_id: String,
    aws_access_secret_key: String,
    aws_access_token: String,
) -> Result<Request<()>> {
    let signable_request = SignableRequest::new(
        "GET",
        url.as_str(),
        std::iter::empty(),
        SignableBody::Bytes(&[]),
    )?;

    let identity = Credentials::new(
        aws_access_key_id,
        aws_access_secret_key,
        Some(aws_access_token),
        None,
        "imds",
    )
    .into();

    let signing_params = SigningParams::builder()
        .identity(&identity)
        .region(aws_region)
        .name("ecs")
        .time(SystemTime::now())
        .settings(SigningSettings::default())
        .build()?
        .into();

    let (signing_instructions, _signature) = sign(signable_request, &signing_params)?.into_parts();

    let mut request = Request::builder()
        .method("GET")
        .uri(url.as_str())
        .header("Host", url.host_str().ok_or(anyhow!("Missing host"))?)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", SEC_WEBSOCKET_VERSION)
        .header(
            "Sec-WebSocket-Key",
            tokio_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .body(())?;

    signing_instructions.apply_to_request_http1x(&mut request);

    Ok(request)
}
