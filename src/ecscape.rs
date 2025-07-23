use crate::{
    ecs_agent_metadata::ECSAgentMetadata,
    ecs_container_instance_registrator::ECSContainerInstanceRegistrator,
    imds_metadata::IMDSMetadata,
};
use anyhow::Result;

pub struct ECScape {
    imds_metadata: IMDSMetadata,
    ecs_agent_metadata: ECSAgentMetadata,
    container_instance_registrator: ECSContainerInstanceRegistrator,
}

impl ECScape {
    pub async fn try_new() -> Result<Self> {
        let imds_metadata = IMDSMetadata::try_new().await?;
        let ecs_agent_metadata = ECSAgentMetadata::try_new(&imds_metadata.local_ip).await?;
        let container_instance_registrator = ECSContainerInstanceRegistrator::try_new().await?;

        Ok(Self {
            imds_metadata,
            ecs_agent_metadata,
            container_instance_registrator,
        })
    }

    pub async fn start(&self) -> Result<()> {
        Ok(())
    }
}
