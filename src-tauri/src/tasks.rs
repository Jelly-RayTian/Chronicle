use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::errors::ChronicleError;

#[derive(Clone, Default)]
pub struct ScanTaskManager {
    cancellations: Arc<Mutex<HashMap<i64, Arc<AtomicBool>>>>,
}

impl ScanTaskManager {
    pub fn register(&self, scan_run_id: i64) -> Result<Arc<AtomicBool>, ChronicleError> {
        let token = Arc::new(AtomicBool::new(false));
        self.cancellations
            .lock()
            .map_err(|_error| ChronicleError::DatabaseState)?
            .insert(scan_run_id, Arc::clone(&token));
        Ok(token)
    }

    pub fn cancel(&self, scan_run_id: i64) -> Result<(), ChronicleError> {
        let cancellations = self
            .cancellations
            .lock()
            .map_err(|_error| ChronicleError::DatabaseState)?;
        let token = cancellations
            .get(&scan_run_id)
            .ok_or(ChronicleError::ScanNotFound)?;
        token.store(true, Ordering::Release);
        Ok(())
    }

    pub fn finish(&self, scan_run_id: i64) {
        if let Ok(mut cancellations) = self.cancellations.lock() {
            cancellations.remove(&scan_run_id);
        }
    }
}
