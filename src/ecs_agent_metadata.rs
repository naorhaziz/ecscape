use anyhow::Result;
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct ECSAgentMetadata {
    pub cluster: String,
    pub container_instance_arn: String,
    pub ecs_agent_version: String,
    pub ecs_agent_hash: String,
}

impl<'de> Deserialize<'de> for ECSAgentMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Raw<'a> {
            cluster: &'a str,
            #[serde(rename = "ContainerInstanceARN")]
            container_instance_arn: &'a str,
            version: &'a str,
        }

        let raw = Raw::deserialize(deserializer)?;

        let version_parts: Vec<&str> = raw.version.splitn(2, '-').collect();
        let ecs_agent_version = version_parts.get(0).unwrap_or(&"").to_string();
        let ecs_agent_hash = version_parts.get(1).unwrap_or(&"").to_string();

        Ok(ECSAgentMetadata {
            cluster: raw.cluster.to_string(),
            container_instance_arn: raw.container_instance_arn.to_string(),
            ecs_agent_version,
            ecs_agent_hash,
        })
    }
}

impl ECSAgentMetadata {
    pub async fn try_new() -> Result<Self> {
        const METADATA_URL: &str = "http://localhost:51678/v1/metadata";

        let metadata = reqwest::get(METADATA_URL).await?.json().await?;
        Ok(metadata)
    }
}
