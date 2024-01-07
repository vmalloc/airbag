//! When working with `Result`s in Rust, one sometimes wants to propagate erroneous results as alerts, but otherwise ignore the erroneous cases.
//!
//! For example, let's assume we have this function
//!
//! ```
//! fn save_to_db(value: u32)
//! {
//!   //...
//! }
//! fn save_metric(value: u32) {
//!     save_to_db(value)
//! }
//! ```
//! Now, let's assume `save_to_db` can fail:
//!
//! ```
//! fn save_to_db(value: u32) -> anyhow::Result<()>
//! {
//!   todo!()
//! }
//! ```
//!
//!  It would not be desirable to terminate our entire
//! app because it failed to save a metric on the one hand, but on the other we cannot ignore this error in production.
//!
//! A simple approach would be to manually report the errors:
//!
//! ```
//! fn save_to_db(value: u32) -> anyhow::Result<()>
//! {
//!   todo!()
//! }
//!
//! fn save_metric(value: u32) {
//!   if let Err(e) = save_to_db(value) {
//!     airbag::trigger(airbag::Alert::builder().title(format!("Error saving to DB: {e:?}")));
//!   }
//! }
//! ```
//!
//! But this would get tedious real fast if we repeat it in every case we need to report such failures.
//!
//! For this purpose Airbag provides the [AirbagResult] trait to help simplify the process. [AirbagResult] is available in `airbag::prelude`:
//! ```
//! use airbag::prelude::*;
//!
//! fn save_to_db(value: u32) -> anyhow::Result<()>
//! {
//!   todo!()
//! }
//!
//! fn save_metric(value: u32) {
//!    let res = save_to_db(value).airbag();
//! }
//! ```
//! The above will report errors if they occur, and return the result unmodified.
//! In cases you want to drop the result (a very common case when ignoring results in Rust code) you can use [AirbagResult::airbag_drop]:
//! ```
//! use airbag::prelude::*;
//!
//! fn save_to_db(value: u32) -> anyhow::Result<()>
//! {
//!   todo!()
//! }
//!
//! fn save_metric(value: u32) {
//!    save_to_db(value).airbag_drop();
//! }
//! ```
//!
//!
//!
//!
pub trait AirbagResult<E>: Sized {
    fn airbag_drop(self) {
        drop(self.airbag())
    }

    fn airbag_drop_with_dedup_key<S: Into<String>, F: Fn() -> S>(self, dedup_key_factory: F) {
        drop(self.airbag_with_dedup_key(dedup_key_factory))
    }

    fn airbag_with_dedup_key<S: Into<String>, F: Fn() -> S>(self, dedup_key_factory: F) -> Self;

    fn airbag_if<F: Fn(&E) -> bool>(self, f: F) -> Self;

    fn airbag(self) -> Self;
}

impl<T, E: std::fmt::Debug + 'static> AirbagResult<E> for Result<T, E> {
    fn airbag(self) -> Self {
        if let Err(e) = &self {
            log::error!("Airbag: handling error {e:?}");
            crate::trigger(crate::alert::Alert::build_error_alert(e));
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

    fn airbag_with_dedup_key<S: Into<String>, F: Fn() -> S>(self, dedup_key_factory: F) -> Self {
        if let Err(e) = &self {
            crate::trigger(
                crate::alert::Alert::build_error_alert(e).dedup_key(dedup_key_factory()),
            );
        }
        self
    }
}
