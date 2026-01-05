//! interruption handling stuff
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone)]
/// an interruption handler
pub struct InterruptHandler {
    /// whether ctrlc has been pressed
    flag: Arc<AtomicBool>,
}

impl InterruptHandler {
    /// make a new interruption handler
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// trigger the handler (set the flag to true)
    pub fn trigger(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// reset the handler if the flag is on
    pub fn check_and_reset(&self) -> bool {
        self.flag.swap(false, Ordering::SeqCst)
    }
}

impl Default for InterruptHandler {
    fn default() -> Self {
        Self::new()
    }
}
