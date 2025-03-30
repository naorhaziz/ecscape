use anyhow::Result;
use reqwest::{
    Client, ClientBuilder, Response,
    header::{HeaderMap, HeaderValue},
};
use serde::Deserialize;

const IMDS_BASE: &str = "http://169.254.169.254";

const IMDS_TOKEN_PATH: &str = "latest/api/token";
const IMDS_LOCAL_IPV4_PATH: &str = "latest/meta-data/local-ipv4";
const IMDS_REGION_PATH: &str = "latest/meta-data/placement/region";
const IMDS_ROLE_NAME_PATH: &str = "latest/meta-data/iam/security-credentials";

const IMDS_ROLE_CREDS_PATH_PREFIX: &str = "latest/meta-data/iam/security-credentials/";

const HEADER_TOKEN_TTL: &str = "X-aws-ec2-metadata-token-ttl-seconds";
const HEADER_TOKEN_TTL_VALUE: &str = "21600";
const HEADER_METADATA_TOKEN: &str = "X-aws-ec2-metadata-token";

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
    pub aws_region: String,
    pub aws_access_key_id: String,
    pub aws_access_secret_key: String,
    pub aws_access_token: String,
}

impl IMDSMetadata {
    pub async fn try_new() -> Result<Self> {
        let client = Self::build_client().await?;

        let local_ip = Self::imds_get(&client, IMDS_LOCAL_IPV4_PATH)
            .await?
            .text()
            .await?;

        let aws_region = Self::imds_get(&client, IMDS_REGION_PATH)
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
            aws_region,
            aws_access_key_id: creds.access_key_id,
            aws_access_secret_key: creds.secret_access_key,
            aws_access_token: creds.token,
        })
    }

    fn imds_url(path: &str) -> String {
        format!("{}/{}", IMDS_BASE, path)
    }

    async fn imds_get(client: &Client, path: &str) -> Result<Response> {
        let response = client
            .get(Self::imds_url(path))
            .send()
            .await?
            .error_for_status()?;
        Ok(response)
    }

    async fn build_client() -> Result<Client> {
        let token = Client::new()
            .put(Self::imds_url(IMDS_TOKEN_PATH))
            .header(HEADER_TOKEN_TTL, HEADER_TOKEN_TTL_VALUE)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let mut headers = HeaderMap::new();
        headers.insert(HEADER_METADATA_TOKEN, HeaderValue::from_str(&token)?);

        let client = ClientBuilder::new().default_headers(headers).build()?;

        Ok(client)
    }
}
