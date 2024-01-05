use httpmock::prelude::*;
use serde_json::json;

const TOKEN: &str = "token here";

#[test]
fn test_pagerduty() {
    let (server, _guard) = mock_pd();

    let test_summary = "this is a test summary";

    let mock = server.mock(|when, then| {
        when.method(POST).path("/v2/enqueue").json_body(json!({
            "routing_key": TOKEN,
            "event_action": "trigger",
            "payload": {
                "severity": "critical",
                "source": "airbag",
                "summary": test_summary
            }
        }));
        then.status(202);
    });

    airbag::trigger(airbag::Alert::builder().title(test_summary)).wait_processed();

    mock.assert();
}

#[test]
fn test_pagerduty_long_summary() {
    let (server, _guard) = mock_pd();

    let test_summary = (0..1001)
        .map(|i| std::char::from_u32('0' as u32 + (i % 10)).unwrap())
        .collect::<String>();
    let mut truncated_summary = test_summary[..997].to_owned();
    truncated_summary.push_str("...");

    let mock = server.mock(|when, then| {
        when.method(POST).path("/v2/enqueue").json_body(json!({
            "routing_key": TOKEN,
            "event_action": "trigger",
            "payload": {
                "severity": "critical",
                "source": "airbag",
                "summary": truncated_summary,
            }
        }));
        then.status(202);
    });

    airbag::trigger(airbag::Alert::builder().title(test_summary)).wait_processed();

    mock.assert();
}

fn mock_pd() -> (MockServer, airbag::ConfiguredHubGuard) {
    let server = MockServer::start();

    let guard = airbag::configure_thread_local(
        airbag::backends::PagerDuty::builder()
            .token(TOKEN)
            .base_url(server.url(""))
            .build(),
    );

    (server, guard)
}
