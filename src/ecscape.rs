use anyhow::Result;

use crate::{ecs_agent_metadata::ECSAgentMetadata, imds_metadata::IMDSMetadata};

pub struct ECScape {
    imds_metadata: IMDSMetadata,
    ecs_agent_metadata: ECSAgentMetadata,
}

impl ECScape {
    pub async fn try_new() -> Result<Self> {
        let imds_metadata = IMDSMetadata::try_new().await?;
        let ecs_agent_metadata = ECSAgentMetadata::try_new(&imds_metadata.local_ip).await?;

        Ok(Self {
            imds_metadata,
            ecs_agent_metadata,
        })
    }

    pub async fn start(&self) -> Result<()> {
        Ok(())
    }
}
