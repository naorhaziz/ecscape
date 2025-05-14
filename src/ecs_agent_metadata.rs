use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Deserializer};

static ECS_VERSION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"v(?P<ver>[\d\.]+)\s+\(\*(?P<hash>[a-fA-F0-9]+)\)").unwrap());

static ARN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^arn:aws:ecs:(?P<region>[^:]+):(?P<account_id>\d+):container-instance/.+$")
        .unwrap()
});

#[derive(Debug)]
pub struct ECSAgentMetadata {
    pub region: String,
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
            cluster: &'a str,
            container_instance_arn: &'a str,
            version: &'a str,
        }

        let raw = Raw::deserialize(deserializer)?;

        let arn_caps = ARN_RE.captures(raw.container_instance_arn).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "Failed to extract region/account_id from: {}",
                raw.container_instance_arn
            ))
        })?;

        let region = &arn_caps["region"];
        let account_id = &arn_caps["account_id"];
        let cluster_arn = format!(
            "arn:aws:ecs:{}:{}:cluster/{}",
            region, account_id, raw.cluster
        );

        let caps = ECS_VERSION_RE.captures(raw.version).ok_or_else(|| {
            serde::de::Error::custom(format!("Failed to parse version string: {}", raw.version))
        })?;

        let ecs_agent_version = caps["ver"].to_string();
        let ecs_agent_hash = caps["hash"].to_string();

        Ok(ECSAgentMetadata {
            region: region.to_string(),
            cluster_arn,
            container_instance_arn: raw.container_instance_arn.to_string(),
            ecs_agent_version,
            ecs_agent_hash,
        })
    }
}

impl ECSAgentMetadata {
    pub async fn try_new(local_ip: &str) -> Result<Self> {
        let metadata_url = format!("http://{local_ip}:51678/v1/metadata");
        let metadata = reqwest::get(metadata_url).await?.json().await?;

        Ok(metadata)
    }
}
