use std::sync::LazyLock;

pub static VERSION: LazyLock<String> = LazyLock::new(|| {
    option_env!("GITHUB_SHA")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string()
});

pub static ARCH: LazyLock<String> = LazyLock::new(|| {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        arch => arch,
    }
    .to_string()
});
