use anyhow::{Result, anyhow};
use aws_credential_types::Credentials;
use aws_sdk_ecs::operation::discover_poll_endpoint::DiscoverPollEndpointOutput;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings, sign},
    sign::v4::SigningParams,
};
use std::time::SystemTime;
use tokio_tungstenite::tungstenite::http::{Method, Request};
use url::Url;

pub trait RequestBuilder {
    fn build_url(
        &self,
        discover_poll_endpoint_output: DiscoverPollEndpointOutput,
        cluster_arn: &str,
        container_instance_arn: &str,
        agent_version: &str,
        agent_hash: &str,
    ) -> Result<Url>;

    fn build_ws_url(url: &str) -> Result<Url> {
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

    fn build_request(
        &self,
        credentials: Credentials,
        discover_poll_endpoint_output: DiscoverPollEndpointOutput,
        region: &str,
        cluster_arn: &str,
        container_instance_arn: &str,
        agent_version: &str,
        agent_hash: &str,
    ) -> Result<Request<()>> {
        let url = self.build_url(
            discover_poll_endpoint_output,
            cluster_arn,
            container_instance_arn,
            agent_version,
            agent_hash,
        )?;

        let signable_request = SignableRequest::new(
            "GET",
            url.as_str(),
            std::iter::empty(),
            SignableBody::Bytes(&[]),
        )?;

        let identity = credentials.into();

        let signing_params = SigningParams::builder()
            .identity(&identity)
            .region(region)
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
}
