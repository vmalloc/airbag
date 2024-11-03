//! A Rust library to simplify and streamline incident reporting to various 3rd party services
//! # Features
//!
//! * Support for multiple configurable backends
//! * [Middleware](middleware) support, allowing applications to customize emitted alerts before they are being sent
//! * Supports [shortcuts](result) for handling `Result`s with propagation to alerts
//! * Catches and reports panics (only when configured globally
//!
//! # Getting Started
//! You configure airbag by using the [](configure) or [](configure_thread_local) functions to register either a global or a thread-local Airbag handler respectively.
//!
//! These functions both receive a [Backend implementor](backends) that will be used to actually emit the generated alerts.
//!
//! ```
//! let _guard = airbag::configure(
//!     airbag::backends::SquadCast::builder()
//!       .region("eu")
//!       .token("your SquadCast API token here")
//!       .build());
//! ```
//! <p style="background:rgba(255,181,77,0.16);padding:0.75em;">
//!  <strong>Note:</strong> when in thread-local mode, Airbag does not catch panics, as panic handlers are always a shared resource in Rust
//! </p>
pub mod alert;
pub mod backends;
mod hub;
pub mod middleware;
mod panic_handler;
pub mod prelude;
pub mod result;
mod utils;

pub use alert::Alert;
pub use hub::ConfiguredHubGuard;
pub use hub::{configure, configure_thread_local, ProcessingReceipt};
pub use result::AirbagResult;

pub fn trigger(alert: impl Into<Alert>) -> ProcessingReceipt {
    let alert = alert.into();
    crate::hub::trigger(alert)
}

#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }
    external_doc_test!(include_str!("../README.md"));
}
