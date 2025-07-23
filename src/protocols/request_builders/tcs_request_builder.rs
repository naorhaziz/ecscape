use crate::protocols::request_builders::RequestBuilder;
use anyhow::{Result, anyhow};
use aws_sdk_ecs::operation::discover_poll_endpoint::DiscoverPollEndpointOutput;
use url::Url;

pub struct TCSRequestBuilder {}

impl TCSRequestBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_ws_url(url: &str) -> Result<Url> {
        let mut url = Url::parse(url)?;

        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/ws", url.path()));
        } else {
            url.set_path(&format!("{}ws", url.path()));
        }

        Ok(url)
    }
}

impl RequestBuilder for TCSRequestBuilder {
    fn build_url(
        &self,
        discover_poll_endpoint_output: DiscoverPollEndpointOutput,
        cluster_arn: &str,
        container_instance_arn: &str,
        agent_version: &str,
        agent_hash: &str,
    ) -> Result<Url> {
        let telemetry_endpoint_url = discover_poll_endpoint_output
            .telemetry_endpoint()
            .ok_or(anyhow!("no telemetry endpoint url"))?;

        let mut ws_url = Self::build_ws_url(telemetry_endpoint_url)?;
        ws_url
            .query_pairs_mut()
            .append_pair("agentHash", agent_hash)
            .append_pair("agentVersion", agent_version)
            .append_pair("cluster", cluster_arn)
            .append_pair("containerInstance", container_instance_arn);

        Ok(ws_url)
    }
}
