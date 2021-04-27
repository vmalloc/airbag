use anyhow::Context;

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
    airbag::configure_pagerduty(
        std::env::var("INTEGRATION_KEY").expect("INTEGRATION_KEY not specified"),
    );

    f().expect("Panicking!");
}
