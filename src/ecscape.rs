use anyhow::Result;
use tracing::debug;

use crate::ecs_agent_metadata::ECSAgentMetadata;

pub struct ECSCape {
    ecs_agent_metadata: ECSAgentMetadata,
}

impl ECSCape {
    pub async fn try_new() -> Result<Self> {
        debug!("fetching ecs agent metadata...");
        let ecs_agent_metadata = ECSAgentMetadata::try_new().await?;

        Ok(Self { ecs_agent_metadata })
    }

    pub async fn start(&self) -> Result<()> {
        Ok(())
    }
}
