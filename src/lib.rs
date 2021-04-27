use lazy_static::lazy_static;
use pagerduty_rs::eventsv2sync::*;
use pagerduty_rs::types::*;
use parking_lot::RwLock;
use std::{fmt::Debug, sync::Arc};

mod panic_handler;
pub mod prelude;

const MAX_SUMMARY_LENGTH: usize = 1000;

lazy_static! {
    pub(crate) static ref PAGERDUTY_INTEGRATION_KEY: Arc<RwLock<Option<String>>> =
        Arc::new(RwLock::new(None));
}

pub fn configure_pagerduty(integration_key: impl Into<String>) {
    PAGERDUTY_INTEGRATION_KEY
        .write()
        .replace(integration_key.into());
    crate::panic_handler::install();
}

pub trait AirbagResult: Sized {
    fn airbag_swallow(self) {
        drop(self.airbag())
    }

    fn airbag(self) -> Self;
}

impl<T, E: Debug + 'static> AirbagResult for Result<T, E> {
    fn airbag(self) -> Self {
        if let Err(e) = &self {
            let integration_key = PAGERDUTY_INTEGRATION_KEY.read().clone();
            if let Some(key) = integration_key {
                let e = generate_error_event(e);
                dispatch_pagerduty_event(key, e);
            }
        }
        self
    }
}

fn generate_error_event(e: &(impl Debug + 'static)) -> Event<String> {
    let e_any: &dyn std::any::Any = e;
    let e_dbg = format!("{:?}", e);
    let (message, description, dedup_key) = if let Some(e) = e_any.downcast_ref::<anyhow::Error>() {
        (e.to_string(), e_dbg.clone(), Some(sha256(&e_dbg)))
    } else {
        (e_dbg.clone(), e_dbg, None)
    };
    Event::AlertTrigger(AlertTrigger {
        payload: AlertTriggerPayload {
            severity: Severity::Error,
            summary: message,
            source: description.clone(),
            timestamp: None,
            component: None,
            group: None,
            class: None,
            custom_details: Some(description),
        },
        dedup_key,
        images: None,
        links: None,
        client: None,
        client_url: None,
    })
}

fn sha256(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn dispatch_pagerduty_event(key: String, mut e: Event<String>) {
    if let Event::AlertTrigger(t) = &mut e {
        if t.payload.summary.len() > MAX_SUMMARY_LENGTH {
            t.payload.summary.truncate(MAX_SUMMARY_LENGTH - 3);
            t.payload.summary.push_str("...");
        }
    }
    let _ = std::thread::spawn(move || {
        let ev2 = EventsV2::new(key, Some("airbag".to_owned())).unwrap();
        if let Err(e) = ev2.event(e) {
            log::error!("Unable to send alert to PagerDuty: {:?}", e);
        }
    })
    .join()
    .map_err(|_| log::error!("Thread panic detected"));
}
