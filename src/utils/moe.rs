use std::sync::atomic::{AtomicBool, Ordering};

static MOE_MODE: AtomicBool = AtomicBool::new(false);

pub fn enable_moe() {
    MOE_MODE.store(true, Ordering::SeqCst);
}

pub fn disable_moe() {
    MOE_MODE.store(false, Ordering::SeqCst);
}

pub fn is_moe() -> bool {
    MOE_MODE.load(Ordering::SeqCst)
}
