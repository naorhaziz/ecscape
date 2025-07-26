use super::{CredentialsReactor, ECScapeCredentials};
use anyhow::Result;
use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Region as S3Region;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials as S3Credentials};
use tracing::debug;

pub struct S3Reactor {
    s3_bucket: Option<String>,
    s3_region: Option<String>,
}

impl S3Reactor {
    pub fn new(s3_bucket: Option<String>, s3_region: Option<String>) -> Self {
        Self {
            s3_bucket,
            s3_region,
        }
    }
}

#[async_trait]
impl CredentialsReactor for S3Reactor {
    async fn react(&self, credentials: ECScapeCredentials) -> Result<()> {
        let Some(s3_bucket) = &self.s3_bucket else {
            debug!("S3 bucket not configured, skipping S3 reactor");
            return Ok(());
        };

        let Some(s3_region) = &self.s3_region else {
            debug!("S3 region not configured, skipping S3 reactor");
            return Ok(());
        };

        let creds = S3Credentials::new(
            &credentials.access_key_id,
            &credentials.secret_access_key,
            Some(credentials.session_token),
            None,
            "ACS",
        );

        let region = S3Region::new(s3_region.clone());
        let config = S3ConfigBuilder::new()
            .region(region)
            .credentials_provider(creds)
            .build();

        let s3 = S3Client::from_conf(config);

        s3.delete_bucket().bucket(s3_bucket).send().await?;

        Ok(())
    }
}
