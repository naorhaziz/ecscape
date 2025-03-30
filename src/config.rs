use std::sync::LazyLock;

pub static VERSION: LazyLock<&'static str> = LazyLock::new(|| {
    option_env!("GIT_COMMIT_HASH")
        .or(option_env!("CI_COMMIT_SHA"))
        .unwrap_or(env!("CARGO_PKG_VERSION"))
});

pub static ARCH: LazyLock<&'static str> = LazyLock::new(|| match std::env::consts::ARCH {
    "x86_64" => "amd64",
    "aarch64" => "arm64",
    other => other,
});

pub const ACS_PROTOCOL_VERSION: &str = "2";
pub const ACS_PROTOCOL_SEC_NUM: &str = "1";
pub const ACS_PROTOCOL_SEND_CREDENTIALS: bool = true;

pub const SEC_WEBSOCKET_VERSION: &str = "13";

pub const DOCKER_VERSION: &str = "25.0.6";
