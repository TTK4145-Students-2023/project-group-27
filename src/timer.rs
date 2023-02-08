use std::time::{Instant, Duration};
use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
    static ref TIMER_END_TIME: Mutex<Instant> = Mutex::new(Instant::now());
    static ref TIMER_ACTIVE: Mutex<bool> = Mutex::new(false);
}

pub fn timer_start(duration: f64) {
    *TIMER_END_TIME.lock()    = Instant::now() + Duration::from_secs_f64(duration);
    *TIMER_ACTIVE.lock()      = true;
}

pub fn timer_stop() {
    *TIMER_ACTIVE.lock()      = false;
}

pub fn timer_timed_out() -> bool {
    return *TIMER_ACTIVE.lock() && Instant::now() > *TIMER_END_TIME.lock();
}
