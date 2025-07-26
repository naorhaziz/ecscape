use anyhow::{Result, anyhow};
use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sdk_secretsmanager::{Client as SecretsManagerClient, config::SharedCredentialsProvider};
use aws_types::{SdkConfig, region::Region};
use tracing::{debug, info};

use super::{CredentialsReactor, ECScapeCredentials};

pub struct SecretsManagerReactor {
    pub region: Region,
    pub secret_arn: Option<String>,
}

impl SecretsManagerReactor {
    pub fn new(secret_arn: Option<String>, region: String) -> Self {
        Self {
            region: Region::new(region),
            secret_arn,
        }
    }
}

#[async_trait]
impl CredentialsReactor for SecretsManagerReactor {
    async fn react(&self, credentials: ECScapeCredentials) -> Result<()> {
        let Some(secret_arn) = &self.secret_arn else {
            debug!("Secret ARN not configured, skipping Secrets Manager reactor");
            return Ok(());
        };

        let aws_credentials = Credentials::new(
            &credentials.access_key_id,
            &credentials.secret_access_key,
            Some(credentials.session_token.clone()),
            None,
            "ACS",
        );

        let credentials_provider = SharedCredentialsProvider::new(aws_credentials);
        let config = SdkConfig::builder()
            .credentials_provider(credentials_provider)
            .region(self.region.clone())
            .build();
        let client = SecretsManagerClient::new(&config);

        let result = client
            .get_secret_value()
            .secret_id(secret_arn.clone())
            .send()
            .await?;

        let secret_value = result
            .secret_string()
            .ok_or(anyhow!("Secret value is empty or not a string"))?;

        info!("Read secret {}: {}", secret_arn, secret_value);

        Ok(())
    }
}
