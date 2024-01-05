use std::sync::Arc;

use parking_lot::{Condvar, Mutex};

pub(crate) fn sha256(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Clone)]
pub(crate) struct DeliverReceipt {
    pair: Arc<(Mutex<bool>, Condvar)>,
}

impl Default for DeliverReceipt {
    fn default() -> Self {
        Self {
            pair: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }
}

impl DeliverReceipt {
    pub(crate) fn wait(&self) {
        let mut m = self.pair.0.lock();
        if !*m {
            self.pair.1.wait(&mut m);
        }
    }

    pub(crate) fn signal(&self) {
        let mut m = self.pair.0.lock();
        *m = true;
        self.pair.1.notify_all();
    }
}
