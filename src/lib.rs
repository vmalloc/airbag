#![deny(warnings)]
#![deny(clippy::dbg_macro)]
use std::fmt::Debug;

mod alerts;
mod dispatch;
mod panic_handler;
pub mod prelude;
mod utils;

pub use dispatch::configure_pagerduty;
use serde_json::Value;

pub trait AirbagResult<E>: Sized {
    fn airbag_drop(self) {
        drop(self.airbag())
    }

    fn airbag_if<F: Fn(&E) -> bool>(self, f: F) -> Self;

    fn airbag(self) -> Self;
}

impl<T, E: Debug + 'static> AirbagResult<E> for Result<T, E> {
    fn airbag(self) -> Self {
        if let Err(e) = &self {
            crate::dispatch::HUB
                .read()
                .dispatch(|| crate::alerts::generate_error_alert(e));
        }
        self
    }

    fn airbag_if<F: Fn(&E) -> bool>(self, f: F) -> Self {
        if let Err(e) = &self {
            if f(e) {
                return self.airbag();
            }
        }
        self
    }
}

pub fn create_alert(summary: impl Into<String>, details: Option<Value>, dedup_key: Option<String>) {
    let summary = summary.into();
    crate::dispatch::HUB.read().dispatch(move || {
        crate::alerts::generate_message_alert(summary.clone(), details.clone(), dedup_key.clone())
    });
}
