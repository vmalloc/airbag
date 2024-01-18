//! Airbag supports installing [Middleware] implementors on backends to allow processing alerts
//! before they are being sent.
//!
//! Installing a middleware is done using the [Backend::wrap] method.
//!
//! For example: this will install a PagerDuty backend that will prefix all alert titles with `"Prefix: "`:
//!
//! ```
//! use airbag::prelude::*;
//!
//! airbag::configure(
//!   airbag::backends::PagerDuty::builder().token("PD token").build().wrap(
//!     airbag::middleware::TitlePrefix::new("Prefix: ")
//!   )
//! );
//! ```
//!
//! Most use cases should probably opt for the [Backend::map] method, which allows wrapping backends in middleware that processes
//! alerts via callbacks. For example, here is a use case that adds a constant field to all alerts
//!
//! ```
//! use airbag::prelude::*;
//!
//! airbag::configure(
//!   airbag::backends::PagerDuty::builder().token("PD token").build().map(
//!      |alert| alert.with_field_if_missing("my_label", "some_value")
//!   )
//! );
//! ```
//!
//!
//!
use crate::backends::Backend;

pub trait Middleware {
    fn process(&mut self, alert: crate::alert::Alert) -> crate::alert::Alert;
}

pub struct Wrapped<B: Backend, M: Middleware> {
    pub(crate) backend: B,
    pub(crate) middleware: M,
}

impl<B, M> Backend for Wrapped<B, M>
where
    B: Backend,
    M: Middleware,
{
    fn send(&mut self, alert: crate::Alert) -> anyhow::Result<()> {
        self.backend.send(self.middleware.process(alert))
    }
}

pub use crate::alert::middleware::DedupKeyPrefix;
pub use crate::alert::middleware::TitlePrefix;

pub struct Map<F>
where
    F: Fn(crate::alert::Alert) -> crate::alert::Alert,
{
    f: F,
}

impl<F> Map<F>
where
    F: Fn(crate::alert::Alert) -> crate::alert::Alert,
{
    pub(crate) fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> Middleware for Map<F>
where
    F: Fn(crate::alert::Alert) -> crate::alert::Alert,
{
    fn process(&mut self, alert: crate::alert::Alert) -> crate::alert::Alert {
        (self.f)(alert)
    }
}
