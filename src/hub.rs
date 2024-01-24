use std::{
    cell::RefCell,
    sync::{Arc, OnceLock},
};

use parking_lot::Mutex;

use crate::{backends::Backend, middleware::Middleware};

static GLOBAL_HUB: OnceLock<Hub> = OnceLock::new();

thread_local! {
    static TL_HUB: RefCell<Option<Hub>> = RefCell::new(None);
}

pub(crate) fn trigger(mut alert: crate::alert::Alert) -> ProcessingReceipt {
    let receipt = ProcessingReceipt::default();
    if let Some(dispatch) = get_backend() {
        log::debug!("Triggering alert #{}", alert.id());
        let middlewares = dispatch.middleware.lock().clone();
        for callback in middlewares {
            alert = callback.process(alert);
        }
        let _ = dispatch
            .sender
            .send(HubMessage::Alert(alert, receipt.clone()));
    } else {
        log::debug!("No hub is configured");
    }
    receipt
}

#[derive(Default)]
pub(crate) struct Hub {
    dispatch: parking_lot::Mutex<Option<HubDispatch>>,
}

pub(crate) enum HubMessage {
    Alert(crate::Alert, ProcessingReceipt),
    Terminate(ProcessingReceipt),
}

#[derive(Clone)]
pub(crate) struct HubDispatch {
    sender: crossbeam::channel::Sender<HubMessage>,
    middleware: Arc<Mutex<Vec<Arc<dyn Middleware + Send + Sync + 'static>>>>,
}

pub(crate) fn get_backend() -> Option<HubDispatch> {
    TL_HUB.with(|hub| {
        let hub = hub.borrow();

        hub.as_ref()
            .and_then(|hub| hub.dispatch.lock().clone())
            .or_else(|| {
                GLOBAL_HUB
                    .get_or_init(Default::default)
                    .dispatch
                    .lock()
                    .clone()
            })
    })
}

pub fn configure<B: Backend + Send + 'static>(backend: B) -> ConfiguredHubGuard {
    let dispatch = spawn_backend(backend);
    let global_hub = GLOBAL_HUB.get_or_init(Default::default);
    global_hub.dispatch.lock().replace(dispatch.clone());
    crate::panic_handler::install();
    ConfiguredHubGuard { dispatch }
}

pub fn configure_thread_local<B: Backend + Send + 'static>(backend: B) -> ConfiguredHubGuard {
    let dispatch = spawn_backend(backend);
    TL_HUB.with(|hub| {
        let mut hub = hub.borrow_mut();
        assert!(
            hub.is_none(),
            "Attempted to configure an already-configured thread-local hub"
        );
        hub.replace(Hub {
            dispatch: parking_lot::Mutex::new(Some(dispatch.clone())),
        });
    });

    ConfiguredHubGuard { dispatch }
}

fn spawn_backend<B: Backend + Send + 'static>(mut backend: B) -> HubDispatch {
    let (sender, receiver) = crossbeam::channel::bounded(1024);
    std::thread::spawn(move || {
        while let Ok(msg) = receiver.recv() {
            match msg {
                HubMessage::Alert(alert, receipt) => {
                    let res = backend.send(alert);
                    if let Err(e) = res {
                        log::error!("Failed sending alert: {e:?}");
                    }
                    receipt.mark_processed();
                }
                HubMessage::Terminate(receipt) => {
                    log::debug!("Backend received termination signal");
                    receipt.mark_processed();
                    break;
                }
            }
        }

        log::debug!("Backend thread terminating")
    });

    HubDispatch {
        sender,
        middleware: Default::default(),
    }
}

pub struct ConfiguredHubGuard {
    dispatch: HubDispatch,
}

impl ConfiguredHubGuard {
    /// installs a middleware to the configured hub
    pub fn install<M: Middleware + Send + Sync + 'static>(self, middleware: M) -> Self {
        self.dispatch.middleware.lock().push(Arc::new(middleware));
        self
    }

    /// Installs a middleware that maps a function over triggered alerts
    pub fn map<F: Fn(crate::Alert) -> crate::Alert + Send + Sync + 'static>(self, f: F) -> Self
    where
        Self: Sized,
    {
        self.install(crate::middleware::Map::new(f))
    }
}

impl Drop for ConfiguredHubGuard {
    fn drop(&mut self) {
        let receipt = ProcessingReceipt::default();

        let _ = self
            .dispatch
            .sender
            .send(HubMessage::Terminate(receipt.clone()));

        log::debug!("Flushing airbag alerts...");
        receipt.wait_processed();
    }
}

#[derive(Default, Clone)]
pub struct ProcessingReceipt {
    cond: Arc<(parking_lot::Mutex<bool>, parking_lot::Condvar)>,
}

impl ProcessingReceipt {
    pub(crate) fn mark_processed(&self) {
        let mut locked = self.cond.0.lock();
        *locked = true;
        self.cond.1.notify_all();
    }

    pub fn wait_processed(&self) {
        let mut finished = self.cond.0.lock();
        while !*finished {
            self.cond.1.wait(&mut finished);
        }
    }
}
