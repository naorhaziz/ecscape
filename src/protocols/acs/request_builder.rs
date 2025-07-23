use anyhow::{Result, anyhow};
use aws_sdk_ecs::operation::discover_poll_endpoint::DiscoverPollEndpointOutput;
use std::sync::atomic::{AtomicBool, Ordering};
use url::Url;

use crate::protocols::request_builder::RequestBuilder;

pub struct ACSRequestBuilder {
    send_credentials: AtomicBool,
}

impl ACSRequestBuilder {
    pub fn new() -> Self {
        Self {
            send_credentials: AtomicBool::new(true),
        }
    }
}

impl RequestBuilder for ACSRequestBuilder {
    fn build_url(
        &self,
        discover_poll_endpoint_output: DiscoverPollEndpointOutput,
        cluster_arn: &str,
        container_instance_arn: &str,
        agent_version: &str,
        agent_hash: &str,
    ) -> Result<Url> {
        // ACS protocol version spec:
        // 1: default protocol version
        // 2: ACS will proactively close the connection when heartbeat ACKs are missing
        pub const ACS_PROTOCOL_VERSION: &str = "1";
        pub const ACS_PROTOCOL_SEC_NUM: &str = "1";

        let poll_endpoint_url = discover_poll_endpoint_output
            .endpoint()
            .ok_or(anyhow!("no acs endpoint url"))?;

        let mut ws_url = Self::build_ws_url(poll_endpoint_url)?;
        ws_url
            .query_pairs_mut()
            .append_pair("agentHash", agent_hash)
            .append_pair("agentVersion", agent_version)
            .append_pair("clusterArn", cluster_arn)
            .append_pair("containerInstanceArn", container_instance_arn)
            .append_pair("protocolVersion", ACS_PROTOCOL_VERSION)
            .append_pair("seqNum", ACS_PROTOCOL_SEC_NUM)
            .append_pair(
                "sendCredentials",
                &self.send_credentials.load(Ordering::Relaxed).to_string(),
            );

        self.send_credentials.store(false, Ordering::Relaxed); // Only send credentials on the first connection

        Ok(ws_url)
    }
}
