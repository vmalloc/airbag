use crate::Alert;

pub mod pagerduty;
pub mod squadcast;

pub use pagerduty::PagerDuty;
pub use squadcast::SquadCast;

pub trait Backend {
    fn send(&mut self, alert: Alert) -> anyhow::Result<()>;

    /// wraps this backend with a middleware that transforms alerts using a provided function
    ///
    fn map<F: Fn(Alert) -> Alert>(
        self,
        f: F,
    ) -> crate::middleware::Wrapped<Self, crate::middleware::Map<F>>
    where
        Self: Sized,
    {
        self.wrap(crate::middleware::Map::new(f))
    }

    /// wraps this backend with a specified middleware
    ///
    fn wrap<M: crate::middleware::Middleware>(
        self,
        middleware: M,
    ) -> crate::middleware::Wrapped<Self, M>
    where
        Self: Sized,
    {
        crate::middleware::Wrapped {
            backend: self,
            middleware,
        }
    }
}
