use crate::Alert;

pub mod pagerduty;
pub mod squadcast;

pub use pagerduty::PagerDuty;
pub use squadcast::SquadCast;

pub trait Backend {
    fn send(&mut self, alert: Alert) -> anyhow::Result<()>;

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
