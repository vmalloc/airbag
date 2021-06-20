use crate::utils::sha256;
use serde_json::{json, Value};
use std::{fmt::Debug, panic::PanicInfo};

const MAX_SUMMARY_LENGTH: usize = 1000;

pub(crate) fn generate_message_alert(
    summary: impl Into<String>,
    details: Option<Value>,
    dedup_key: Option<String>,
) -> Value {
    generate_alert(summary.into(), "Airbag".into(), details, dedup_key)
}

pub(crate) fn generate_error_alert(e: &(impl Debug + 'static), dedup_key: Option<String>) -> Value {
    let e_any: &dyn std::any::Any = e;
    let e_dbg = format!("{:?}", e);
    let (summary, source, dedup_key) = if let Some(e) = e_any.downcast_ref::<anyhow::Error>() {
        (
            e.to_string(),
            e_dbg.clone(),
            dedup_key.or_else(|| Some(sha256(&e_dbg))),
        )
    } else {
        (e_dbg.clone(), e_dbg, dedup_key)
    };

    generate_alert(
        summary,
        source,
        Some(Value::String(format!("{:?}", e))),
        dedup_key,
    )
}

pub(crate) fn generate_panic_alert(info: &PanicInfo) -> Value {
    let location = if let Some(location) = info.location() {
        format!("{:?}", location)
    } else {
        String::from("<unknown>")
    };
    let summary = format!(
        "Panic at {}: {}",
        location,
        info.payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .unwrap_or("N/A")
    );

    let dedup_key = Some(sha256(&summary));

    generate_alert(
        summary.clone(),
        location,
        Some(Value::String(summary)),
        dedup_key,
    )
}

fn generate_alert(
    mut summary: String,
    source: String,
    details: Option<Value>,
    dedup_key: Option<String>,
) -> Value {
    if summary.len() > MAX_SUMMARY_LENGTH {
        summary.truncate(MAX_SUMMARY_LENGTH - 3);
        summary.push_str("...");
    }
    json!({
        "event_action": "trigger",
        "dedup_key": dedup_key,
        "payload": {
            "summary": summary,
            "source": source,
            "severity": "error",
            "custom_details": details,
        }
    })
}
