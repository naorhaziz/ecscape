use anyhow::{Result, anyhow};
use aws_credential_types::Credentials;
use aws_sdk_ecs::{Client as EcsClient, config::SharedCredentialsProvider};
use aws_types::{SdkConfig, region::Region};
use tracing::debug;
use url::Url;

use crate::{
    ecs_agent_metadata::ECSAgentMetadata, imds_metadata::IMDSMetadata, utils::build_ws_url,
    ws_client::WSClient,
};

pub struct ACSConnector {
    imds_metadata: IMDSMetadata,
    ecs_agent_metadata: ECSAgentMetadata,
}

impl ACSConnector {
    pub async fn try_new() -> Result<Self> {
        let imds_metadata = IMDSMetadata::try_new().await?;
        let ecs_agent_metadata = ECSAgentMetadata::try_new(&imds_metadata.local_ip).await?;

        Ok(Self {
            imds_metadata,
            ecs_agent_metadata,
        })
    }

    pub async fn obtain_poll_endpoint_url(
        &self,
        send_credentials: bool,
        credentials: Credentials,
    ) -> Result<Url> {
        // ACS protocol version spec:
        // 1: default protocol version
        // 2: ACS will proactively close the connection when heartbeat ACKs are missing
        pub const ACS_PROTOCOL_VERSION: &str = "1";
        pub const ACS_PROTOCOL_SEC_NUM: &str = "1";
        pub const DOCKER_VERSION: &str = "25.0.8";

        let credentials_provider = SharedCredentialsProvider::new(credentials);

        let region = Region::new(self.ecs_agent_metadata.region.clone());

        let sdk_config = SdkConfig::builder()
            .credentials_provider(credentials_provider)
            .region(region)
            .build();

        let ecs_client = EcsClient::new(&sdk_config);

        let discover_poll_endpoint_output = ecs_client
            .discover_poll_endpoint()
            .cluster(&self.ecs_agent_metadata.cluster_arn)
            .container_instance(&self.ecs_agent_metadata.container_instance_arn)
            .send()
            .await?;

        let poll_endpoint_url = discover_poll_endpoint_output
            .endpoint()
            .ok_or(anyhow!("no acs endpoint url"))?;

        let mut ws_url = build_ws_url(poll_endpoint_url)?;
        ws_url
            .query_pairs_mut()
            .append_pair("agentHash", &self.ecs_agent_metadata.ecs_agent_hash)
            .append_pair("agentVersion", &self.ecs_agent_metadata.ecs_agent_version)
            .append_pair("clusterArn", &self.ecs_agent_metadata.cluster_arn)
            .append_pair(
                "containerInstanceArn",
                &self.ecs_agent_metadata.container_instance_arn,
            )
            .append_pair("dockerVersion", DOCKER_VERSION)
            .append_pair("protocolVersion", ACS_PROTOCOL_VERSION)
            .append_pair("seqNum", ACS_PROTOCOL_SEC_NUM)
            .append_pair("sendCredentials", &send_credentials.to_string());

        Ok(ws_url)
    }

    pub async fn connect(&self, send_credentials: bool) -> Result<WSClient> {
        let credentials = Credentials::new(
            self.imds_metadata.aws_access_key_id.as_str(),
            self.imds_metadata.aws_access_secret_key.as_str(),
            Some(self.imds_metadata.aws_access_token.clone()),
            None,
            "IMDS",
        );

        let poll_endpoint_url = self
            .obtain_poll_endpoint_url(send_credentials, credentials.clone())
            .await?;
        debug!("ACS Poll Endpoint url: {:?}", poll_endpoint_url);

        let ws_client = WSClient::connect_with_sigv4(
            poll_endpoint_url,
            &self.ecs_agent_metadata.region,
            credentials,
        )
        .await?;

        Ok(ws_client)
    }
}
