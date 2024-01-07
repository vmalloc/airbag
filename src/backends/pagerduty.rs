use anyhow::Context;
use reqwest::Url;
use serde_json::json;

use crate::{alert::AlertMeta, utils::json_set_if_not_present};

const MAX_SUMMARY_LENGTH: usize = 1000;

/// The `PagerDuty` struct implements a backend for the [PagerDuty](https://pagerduty.com) service, and uses its
/// "Events v2" integration API to emit alerts
#[derive(typed_builder::TypedBuilder)]
pub struct PagerDuty {
    #[builder(setter(into))]
    token: String,

    #[builder(default, setter(strip_option))]
    base_url: Option<String>,
}

impl super::Backend for PagerDuty {
    fn send(&mut self, alert: crate::alert::Alert) -> anyhow::Result<()> {
        let url = Url::parse(
            self.base_url
                .as_deref()
                .unwrap_or("https://events.pagerduty.com"),
        )
        .context("Cannot parse URL")?
        .join("/v2/enqueue")
        .unwrap();

        let mut json = alert.as_json().clone();

        if json["payload"]["source"].is_null() {
            json["payload"]["source"] = json!("airbag");
        }

        json["routing_key"] = serde_json::json!(self.token.clone());

        let AlertMeta {
            title,
            dedup_key,
            severity,
            priority,
            description,
        } = alert.meta();

        let severity =
            severity.unwrap_or_else(|| priority.unwrap_or(crate::alert::Priority::P1).into());

        json_set_if_not_present(
            &mut json,
            &["payload", "severity"],
            match severity {
                crate::alert::Severity::Critical => "critical",
                crate::alert::Severity::Error => "error",
                crate::alert::Severity::Warning => "warning",
                crate::alert::Severity::Info => "info",
            },
        );

        json_set_if_not_present(
            &mut json,
            &["payload", "summary"],
            title.as_deref().unwrap_or("Airbag alert"),
        );

        if let Some(description) = description {
            json_set_if_not_present(&mut json, &["payload", "custom_details"], description);
        }

        if let Some(dedup_key) = dedup_key {
            json["dedup_key"] = json!(dedup_key);
        }

        json["event_action"] = json!("trigger");

        if json["payload"]["summary"].as_str().unwrap_or("").len() > MAX_SUMMARY_LENGTH {
            let mut summary = json["payload"]["summary"].take().as_str().unwrap()
                [..MAX_SUMMARY_LENGTH - 3]
                .to_owned();
            summary.push_str("...");

            json["payload"]["summary"] = json!(summary);
        }

        log::debug!("Pagerduty event: {json:?}");

        log::debug!("Sending alert #{} to PagerDuty", alert.id());
        crate::utils::http_retry(|client| client.post(url.clone()).json(&json))?;

        Ok(())
    }
}
