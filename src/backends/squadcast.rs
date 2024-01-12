use anyhow::Context;
use reqwest::Url;
use serde_json::json;

use crate::{alert::AlertMeta, utils::json_set_if_not_present};

/// The `SquadCast` struct implements a backend for the [SquadCast](https://squadcast.com) service.
/// Configuration options include SquadCast's region name and webhook token.
///
/// The webhook token is the end component of the incidents v2 webhook URL.
///
/// For example, if your webhook URL is https://api.eu.squadcast.com/v2/incidents/api/7f550a9f4c44173a37664d938f1355f0f92a47a7,
/// then your token is "7f550a9f4c44173a37664d938f1355f0f92a47a7", and your region is "eu", meaning the configuration would be
///
/// ```
/// use airbag::backends::squadcast::SquadCast;
///
/// let backend = SquadCast::builder().token("7f550a9f4c44173a37664d938f1355f0f92a47a7").region("eu").build();
/// ```
#[derive(typed_builder::TypedBuilder)]
pub struct SquadCast {
    #[builder(setter(into))]
    token: String,

    #[builder(setter(into))]
    region: String,

    #[builder(default, setter(strip_option))]
    base_url: Option<String>,
}

impl super::Backend for SquadCast {
    fn send(&mut self, alert: crate::alert::Alert) -> anyhow::Result<()> {
        let url = Url::parse(
            &self
                .base_url
                .as_ref()
                .cloned()
                .unwrap_or_else(|| format!("https://api.{}.squadcast.com", self.region)),
        )
        .context("Cannot parse URL")?
        .join(&format!("/v2/incidents/api/{}", self.token))
        .unwrap();

        let mut json = alert.as_json().clone();

        let AlertMeta {
            title,
            dedup_key,
            severity,
            priority,
            description,
        } = alert.meta();

        json_set_if_not_present(
            &mut json,
            &["priority"],
            match priority
                .unwrap_or_else(|| severity.unwrap_or(crate::alert::Severity::Error).into())
            {
                crate::alert::Priority::P1 => "P1",
                crate::alert::Priority::P2 => "P2",
                crate::alert::Priority::P3 => "P3",
                crate::alert::Priority::P4 => "P4",
                crate::alert::Priority::P5 => "P5",
            },
        );

        json_set_if_not_present(
            &mut json,
            &["message"],
            title.as_deref().unwrap_or("Airbag alert"),
        );

        if let Some(description) = description {
            json_set_if_not_present(&mut json, &["description"], description);
        }

        if let Some(dedup_key) = dedup_key {
            json["dedup_key"] = json!(dedup_key);
        }

        log::debug!("Squadcast event: {json:?}");

        log::debug!("Sending alert #{} to SquadCast", alert.id());
        crate::utils::http_retry(|client| client.post(url.clone()).json(&json))?;

        Ok(())
    }
}
