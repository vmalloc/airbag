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
