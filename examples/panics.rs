use anyhow::Context;
use clap::Parser;

fn f() -> anyhow::Result<()> {
    g().context("Calling g")
}

fn g() -> anyhow::Result<()> {
    h().context("Calling h")
}

fn h() -> anyhow::Result<()> {
    anyhow::bail!("Error here")
}

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    pagerduty_token: Option<String>,

    #[clap(long)]
    squadcast_token: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let opts = Opts::parse();

    let _guard = if let Some(token) = opts.pagerduty_token {
        airbag::configure(
            airbag::backends::pagerduty::PagerDuty::builder()
                .token(token)
                .build(),
        )
    } else if let Some(token) = opts.squadcast_token {
        airbag::configure(
            airbag::backends::squadcast::SquadCast::builder()
                .token(token)
                .region("eu")
                .build(),
        )
    } else {
        anyhow::bail!("No token specified")
    };
    f().expect("Panicking!");
    Ok(())
}
