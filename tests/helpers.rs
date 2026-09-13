//! Shared helpers for integration tests.
//!
//! The library keeps process-global dictionaries (like the JS singletons),
//! so tests that mutate global state take the serial lock and clean up.

use std::sync::{Mutex, MutexGuard};

static SERIAL: Mutex<()> = Mutex::new(());

pub fn lock() -> MutexGuard<'static, ()> {
    // Poison-tolerant: a failing test must not cascade into unrelated ones.
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}
