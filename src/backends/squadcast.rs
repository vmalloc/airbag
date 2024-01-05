use anyhow::Context;
use reqwest::Url;
use serde_json::json;

use crate::{alert::AlertMeta, utils::json_set_if_not_present};

#[derive(typed_builder::TypedBuilder)]
pub struct SquadCast {
    #[builder(setter(into))]
    token: String,

    #[builder(default, setter(strip_option))]
    base_url: Option<String>,
}

impl super::Backend for SquadCast {
    fn send(&mut self, alert: crate::alert::Alert) -> anyhow::Result<()> {
        let url = Url::parse(
            self.base_url
                .as_deref()
                .unwrap_or("https://api.eu.squadcast.com"),
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
