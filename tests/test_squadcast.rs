use httpmock::prelude::*;
use serde_json::json;

const TOKEN: &str = "token-here";

#[test]
fn test_squadcast() {
    tracing_subscriber::fmt::init();
    let (server, _guard) = mock_sc();

    let test_title = "this is a test summary";
    let test_description = "this is a test description";

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/v2/incidents/api/{TOKEN}"))
            .json_body(json!({
                "message": test_title,
                "description": test_description,
                "priority": "P2",
            }));
        then.status(202);
    });

    airbag::trigger(
        airbag::Alert::builder()
            .title(test_title)
            .description(test_description),
    )
    .wait_processed();

    mock.assert();
}

fn mock_sc() -> (MockServer, airbag::ConfiguredHubGuard) {
    let server = MockServer::start();

    let guard = airbag::configure_thread_local(
        airbag::backends::SquadCast::builder()
            .token(TOKEN)
            .base_url(server.url(""))
            .build(),
    );

    (server, guard)
}
