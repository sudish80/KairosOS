use std::sync::atomic::{AtomicU64, Ordering};
static COUNTER: AtomicU64 = AtomicU64::new(0);
pub fn inc() { COUNTER.fetch_add(1, Ordering::Relaxed); }
pub fn count() -> u64 { COUNTER.load(Ordering::Relaxed) }
