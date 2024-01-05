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
