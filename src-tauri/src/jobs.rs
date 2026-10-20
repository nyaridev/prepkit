//! One cancel flag per running tool, so the window's Cancel button can stop a batch.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Jobs {
    flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl Jobs {
    /// A fresh flag for `job`, replacing any earlier one.
    pub fn start(&self, job: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.flags.lock().unwrap().insert(job.to_string(), flag.clone());
        flag
    }

    pub fn cancel(&self, job: &str) {
        if let Some(flag) = self.flags.lock().unwrap().get(job) {
            flag.store(true, Ordering::Relaxed);
        }
    }
}
