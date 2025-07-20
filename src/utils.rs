use anyhow::{Result, anyhow};
use url::Url;

pub fn build_ws_url(url: &str) -> Result<Url> {
    let mut url = Url::parse(url)?;

    match url.scheme() {
        "http" => url
            .set_scheme("ws")
            .map_err(|_| anyhow!("failed to set scheme to ws"))?,
        "https" => url
            .set_scheme("wss")
            .map_err(|_| anyhow!("failed to set scheme to wss"))?,
        scheme => return Err(anyhow!("unsupported scheme: {scheme}")),
    };

    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/ws", url.path()));
    } else {
        url.set_path(&format!("{}ws", url.path()));
    }

    Ok(url)
}
