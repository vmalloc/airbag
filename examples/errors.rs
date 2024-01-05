use airbag::prelude::*;
use anyhow::{Context, Ok};
use clap::Parser;

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    pagerduty_token: Option<String>,

    #[clap(long)]
    squadcast_token: Option<String>,
}

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
                .build(),
        )
    } else {
        anyhow::bail!("No token specified")
    };
    /* let _guard = airbag::configure_pagerduty(
        std::env::var("INTEGRATION_KEY").expect("INTEGRATION_KEY not specified"),
        Some(json!({"id": 10})),
        Some("dedup_prefix".into()),
        Some("Alert prefix here: ".into()),
    ); */

    f().map_err(|e| {
        println!("* {:?}", e);
        e
    })
    .airbag_drop();

    Ok(())
}
