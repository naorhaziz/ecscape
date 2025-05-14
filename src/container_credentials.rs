use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug)]
pub struct ContainerCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub token: String,
}

impl ContainerCredentials {
    pub async fn try_new() -> Result<Self> {
        #[derive(Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Raw {
            access_key_id: String,
            secret_access_key: String,
            token: String,
        }

        let relative_uri = std::env::var("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI")?;
        let url = format!("http://169.254.170.2{relative_uri}");

        let client = Client::new();
        let raw: Raw = client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(Self {
            access_key_id: raw.access_key_id,
            secret_access_key: raw.secret_access_key,
            token: raw.token,
        })
    }
}
