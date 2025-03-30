use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Deserializer};

static ECS_VERSION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"v(?P<ver>[\d\.]+)\s+\(\*(?P<hash>[a-fA-F0-9]+)\)").unwrap());

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

        let caps = ECS_VERSION_RE.captures(raw.version).ok_or_else(|| {
            serde::de::Error::custom(format!("Failed to parse version string: {}", raw.version))
        })?;

        let ecs_agent_version = caps["ver"].to_string();
        let ecs_agent_hash = caps["hash"].to_string();

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
        const INSTANCE_METADATA_IP_URL: &str = "http://169.254.169.254/latest/meta-data/local-ipv4";

        let ip = reqwest::get(INSTANCE_METADATA_IP_URL).await?.text().await?;

        let metadata_url = format!("http://{}:51678/v1/metadata", ip);
        let metadata = reqwest::get(metadata_url).await?.json().await?;

        Ok(metadata)
    }
}
