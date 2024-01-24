use crate::Alert;

pub mod pagerduty;
pub mod squadcast;

pub use pagerduty::PagerDuty;
pub use squadcast::SquadCast;

pub trait Backend {
    fn send(&mut self, alert: Alert) -> anyhow::Result<()>;
}
