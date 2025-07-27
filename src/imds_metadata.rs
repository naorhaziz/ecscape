use anyhow::Result;
use reqwest::{Client, Response};
use serde::Deserialize;

const IMDS_BASE: &str = "http://169.254.169.254";
const IMDS_LOCAL_IPV4_PATH: &str = "latest/meta-data/local-ipv4";
const IMDS_ROLE_NAME_PATH: &str = "latest/meta-data/iam/security-credentials";
const IMDS_ROLE_CREDS_PATH_PREFIX: &str = "latest/meta-data/iam/security-credentials/";
const IMDS_REGION_PATH: &str = "latest/meta-data/placement/region";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ImdsRoleCredentials {
    access_key_id: String,
    secret_access_key: String,
    token: String,
}

#[derive(Debug)]
pub struct IMDSMetadata {
    pub local_ip: String,
    pub region: String,
    pub aws_access_key_id: String,
    pub aws_access_secret_key: String,
    pub aws_access_token: String,
}

impl IMDSMetadata {
    pub async fn try_new() -> Result<Self> {
        let client = Client::new();

        let local_ip = Self::imds_get(&client, IMDS_LOCAL_IPV4_PATH)
            .await?
            .text()
            .await?;

        let region = Self::imds_get(&client, IMDS_REGION_PATH)
            .await?
            .text()
            .await?;

        let role_name = Self::imds_get(&client, IMDS_ROLE_NAME_PATH)
            .await?
            .text()
            .await?;

        let creds_path = format!("{}{}", IMDS_ROLE_CREDS_PATH_PREFIX, role_name.trim());
        let creds: ImdsRoleCredentials = Self::imds_get(&client, &creds_path).await?.json().await?;

        Ok(Self {
            local_ip,
            region,
            aws_access_key_id: creds.access_key_id,
            aws_access_secret_key: creds.secret_access_key,
            aws_access_token: creds.token,
        })
    }

    async fn imds_get(client: &Client, path: &str) -> Result<Response> {
        let response = client
            .get(format!("{}/{}", IMDS_BASE, path))
            .send()
            .await?
            .error_for_status()?;
        Ok(response)
    }
}
