use std::sync::atomic::{AtomicU32, Ordering};

pub static MARVELOUS: AtomicU32 = AtomicU32::new(0);
pub static PERFECT: AtomicU32 = AtomicU32::new(0);
pub static GREAT: AtomicU32 = AtomicU32::new(0);
pub static GOOD: AtomicU32 = AtomicU32::new(0);
pub static MISS: AtomicU32 = AtomicU32::new(0);
pub static OK: AtomicU32 = AtomicU32::new(0);
pub static NG: AtomicU32 = AtomicU32::new(0);
pub static FAST: AtomicU32 = AtomicU32::new(0);
pub static SLOW: AtomicU32 = AtomicU32::new(0);

pub fn reset() {
    MARVELOUS.store(0, Ordering::Relaxed);
    PERFECT.store(0, Ordering::Relaxed);
    GREAT.store(0, Ordering::Relaxed);
    GOOD.store(0, Ordering::Relaxed);
    MISS.store(0, Ordering::Relaxed);
    OK.store(0, Ordering::Relaxed);
    NG.store(0, Ordering::Relaxed);
    FAST.store(0, Ordering::Relaxed);
    SLOW.store(0, Ordering::Relaxed);
}
