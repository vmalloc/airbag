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
