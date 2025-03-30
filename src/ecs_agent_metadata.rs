use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Deserializer};

static ECS_VERSION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"v(?P<ver>[\d\.]+)\s+\(\*(?P<hash>[a-fA-F0-9]+)\)").unwrap());

#[derive(Debug)]
pub struct ECSAgentMetadata {
    pub cluster_arn: String,
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
            #[serde(rename = "ContainerInstanceARN")]
            container_instance_arn: &'a str,
            version: &'a str,
        }

        let raw = Raw::deserialize(deserializer)?;

        // Extract cluster ARN by removing the last path segment from container_instance_arn
        let cluster_arn = raw
            .container_instance_arn
            .rsplitn(2, '/')
            .nth(1)
            .ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "Failed to parse cluster ARN from: {}",
                    raw.container_instance_arn
                ))
            })?
            .to_string();

        let caps = ECS_VERSION_RE.captures(raw.version).ok_or_else(|| {
            serde::de::Error::custom(format!("Failed to parse version string: {}", raw.version))
        })?;

        let ecs_agent_version = caps["ver"].to_string();
        let ecs_agent_hash = caps["hash"].to_string();

        Ok(ECSAgentMetadata {
            cluster_arn,
            container_instance_arn: raw.container_instance_arn.to_string(),
            ecs_agent_version,
            ecs_agent_hash,
        })
    }
}

impl ECSAgentMetadata {
    pub async fn try_new(local_ip: &str) -> Result<Self> {
        let metadata_url = format!("http://{}:51678/v1/metadata", local_ip);
        let metadata = reqwest::get(metadata_url).await?.json().await?;

        Ok(metadata)
    }
}
