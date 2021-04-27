use crate::{dispatch_pagerduty_event, sha256};
use pagerduty_rs::types::{AlertTrigger, AlertTriggerPayload, Event, Severity};
use std::panic::PanicInfo;

pub(crate) fn install() {
    let next = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(integration_key) = crate::PAGERDUTY_INTEGRATION_KEY.read().clone() {
            let event = generate_panic_event(info);
            dispatch_pagerduty_event(integration_key, event);
        }
        next(info);
    }))
}

fn generate_panic_event(info: &PanicInfo) -> Event<String> {
    let location = if let Some(location) = info.location() {
        format!("{:?}", location)
    } else {
        String::from("<unknown>")
    };
    let message = format!(
        "Panic at {}: {}",
        location,
        info.payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .unwrap_or("N/A")
    );

    let dedup_key = Some(sha256(&message));
    Event::AlertTrigger(AlertTrigger {
        payload: AlertTriggerPayload {
            severity: Severity::Error,
            summary: message.clone(),
            source: location,
            timestamp: None,
            component: None,
            group: None,
            class: None,
            custom_details: Some(message),
        },
        dedup_key,
        images: None,
        links: None,
        client: None,
        client_url: None,
    })
}
