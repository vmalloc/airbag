use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use parking_lot::Mutex;

use crate::{backends::Backend, middleware::Middleware};

static GLOBAL_HUB: OnceLock<Hub> = OnceLock::new();

thread_local! {
    static TL_HUB: RefCell<Option<Hub>> = const { RefCell::new(None) };
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

const MIN_INTERVAL_BETWEEN_DUP_ALERTS: std::time::Duration = std::time::Duration::from_secs(5);

fn spawn_backend<B: Backend + Send + 'static>(mut backend: B) -> HubDispatch {
    let (sender, receiver) = crossbeam::channel::bounded(1024);
    std::thread::spawn(move || {
        let mut recent_dedup_keys: HashMap<String, std::time::Instant> = HashMap::new();
        log::debug!("Backend started...");
        while let Ok(msg) = receiver.recv() {
            match msg {
                HubMessage::Alert(alert, receipt) => {
                    let alert_id = alert.id();
                    let now = std::time::Instant::now();

                    let should_send = if let Some(dedup_key) = &alert.meta().dedup_key {
                        recent_dedup_keys.retain(|_, last_sent| {
                            *last_sent >= now - MIN_INTERVAL_BETWEEN_DUP_ALERTS
                        });
                        !recent_dedup_keys.contains_key(dedup_key)
                    } else {
                        true
                    };

                    if should_send {
                        let dedup_key = alert.meta().dedup_key.clone();
                        log::debug!("Backend got alert #{alert_id}. Sending...");
                        let res = backend.send(alert);
                        if let Err(e) = res {
                            log::error!("Failed sending alert: {e:?}");
                        } else {
                            log::debug!("Alert #{alert_id} sent successfully");
                            if let Some(key) = dedup_key {
                                recent_dedup_keys.insert(key, now);
                            }
                        }
                    } else {
                        log::debug!("Skipping sending #{alert_id} - same dedup key sent recently");
                    }
                    receipt.mark_processed();
                }
                HubMessage::Terminate(receipt) => {
                    log::debug!("Backend received termination signal");
                    receipt.mark_processed();
                    break;
                }
            }
            log::debug!("Backend waiting for next message...")
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
    pub fn with_middleware<M: Middleware + Send + Sync + 'static>(self, middleware: M) -> Self {
        self.dispatch.middleware.lock().push(Arc::new(middleware));
        self
    }

    /// Installs a middleware that maps a function over triggered alerts
    pub fn map<F: Fn(crate::Alert) -> crate::Alert + Send + Sync + 'static>(self, f: F) -> Self
    where
        Self: Sized,
    {
        self.with_middleware(crate::middleware::Map::new(f))
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
