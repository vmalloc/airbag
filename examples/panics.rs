use anyhow::Context;
use serde_json::json;

fn f() -> anyhow::Result<()> {
    g().context("Calling g")
}

fn g() -> anyhow::Result<()> {
    h().context("Calling h")
}

fn h() -> anyhow::Result<()> {
    anyhow::bail!("Error here")
}

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Trace)
        .init();
    let _guard = airbag::configure_pagerduty(
        std::env::var("INTEGRATION_KEY").expect("INTEGRATION_KEY not specified"),
        Some(json!({"id": 6})),
    );

    f().expect("Panicking!");
}
