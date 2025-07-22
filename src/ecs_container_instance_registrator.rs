use crate::{ecs_agent_metadata::ECSAgentMetadata, imds_metadata::IMDSMetadata};
use anyhow::{Result, anyhow};
use aws_credential_types::Credentials;
use aws_sdk_ecs::{
    Client as EcsClient,
    config::SharedCredentialsProvider,
    types::builders::{AttributeBuilder, ResourceBuilder, VersionInfoBuilder},
};
use aws_types::{SdkConfig, region::Region};
use sysinfo::System;

pub struct ECSContainerInstanceRegistrator {
    container_instance_arn: String,
}

impl ECSContainerInstanceRegistrator {
    pub async fn try_new() -> Result<Self> {
        const DOCKER_VERSION: &str = "25.0.8";

        let imds_metadata = IMDSMetadata::try_new().await?;
        let ecs_agent_metadata = ECSAgentMetadata::try_new(&imds_metadata.local_ip).await?;

        let credentials = Credentials::new(
            imds_metadata.aws_access_key_id.as_str(),
            imds_metadata.aws_access_secret_key.as_str(),
            Some(imds_metadata.aws_access_token.clone()),
            None,
            "IMDS",
        );

        let credentials_provider = SharedCredentialsProvider::new(credentials);
        let region = Region::new(ecs_agent_metadata.region);
        let sdk_config = SdkConfig::builder()
            .credentials_provider(credentials_provider)
            .region(region)
            .build();
        let ecs_client = EcsClient::new(&sdk_config);

        let num_cpus = num_cpus::get() as i32 * 1024;
        let sys = System::new_all();
        let memory_mb = (sys.total_memory() / 1024 / 1024) as i32;

        let resources = vec![
            ResourceBuilder::default()
                .name("CPU")
                .r#type("INTEGER")
                .integer_value(num_cpus)
                .build(),
            ResourceBuilder::default()
                .name("MEMORY")
                .r#type("INTEGER")
                .integer_value(memory_mb)
                .build(),
            ResourceBuilder::default()
                .name("PORTS")
                .r#type("STRINGSET")
                .set_string_set_value(Some(vec![])) // No reserved ports by default
                .build(),
            ResourceBuilder::default()
                .name("PORTS_UDP")
                .r#type("STRINGSET")
                .set_string_set_value(Some(vec![])) // No reserved UDP ports by default
                .build(),
        ];

        // Build version info with ECS agent metadata and Docker version
        let version_info = VersionInfoBuilder::default()
            .agent_version(ecs_agent_metadata.ecs_agent_version)
            .agent_hash(ecs_agent_metadata.ecs_agent_hash)
            .docker_version(DOCKER_VERSION)
            .build();

        let result = ecs_client
            .register_container_instance()
            .cluster(&ecs_agent_metadata.cluster_arn)
            .instance_identity_document(&imds_metadata.identity_document)
            .instance_identity_document_signature(&imds_metadata.identity_signature)
            .set_total_resources(Some(resources))
            .version_info(version_info)
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.secrets.asm.environment-variables")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.secrets.asm.bootstrap.log-driver")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.secrets.ssm.environment-variables")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.secrets.ssm.bootstrap.log-driver")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.execution-role-awslogs")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.execution-role-ecr-pull")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.container-health-check")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.task-cpu-mem-limit")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.logging-driver.awslogs")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.docker-remote-api.1.17")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.docker-remote-api.1.18")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.docker-remote-api.1.19")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.task-iam-role")
                    .build()?,
            )
            .attributes(
                AttributeBuilder::default()
                    .name("ecs.capability.task-iam-role-network-host")
                    .build()?,
            )
            .send()
            .await?;

        let container_instance = result.container_instance().ok_or(anyhow!(
            "Failed to register container instance: no container instance returned"
        ))?;

        let container_instance_arn = container_instance
            .container_instance_arn()
            .ok_or(anyhow!("Failed to get container instance ARN"))?;

        Ok(Self {
            container_instance_arn: container_instance_arn.to_string(),
        })
    }

    pub fn container_instance_arn(&self) -> &str {
        self.container_instance_arn.as_ref()
    }
}
