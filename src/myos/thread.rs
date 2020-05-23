// Thread

use super::arch::cpu::Cpu;
use alloc::boxed::Box;
use core::ops::*;

static mut TIMER_SOURCE: Option<Box<dyn TimerSource>> = None;

#[allow(dead_code)]
pub struct Thread {
    id: usize,
}

unsafe impl Sync for Thread {}

impl Thread {
    pub fn spawn<F>(f: F)
    where
        F: FnOnce() -> (),
    {
        // TODO: spawn
        f();
    }

    pub fn sleep(duration: TimeMeasure) {
        let timer = unsafe { TIMER_SOURCE.as_ref().unwrap() };
        let deadline = timer.create(duration);
        while timer.until(deadline) {
            unsafe {
                Cpu::halt();
            }
        }
    }

    pub fn usleep(us: u64) {
        Self::sleep(TimeMeasure::from_micros(us));
    }
}

pub trait TimerSource {
    fn create(&self, h: TimeMeasure) -> TimeMeasure;
    fn until(&self, h: TimeMeasure) -> bool;
    fn diff(&self, h: TimeMeasure) -> isize;
}

#[derive(Debug, Copy, Clone)]
pub struct Timer {
    deadline: TimeMeasure,
}

impl Timer {
    pub fn new(duration: TimeMeasure) -> Self {
        let timer = unsafe { TIMER_SOURCE.as_ref().unwrap() };
        Timer {
            deadline: timer.create(duration),
        }
    }

    pub fn until(&self) -> bool {
        let timer = unsafe { TIMER_SOURCE.as_ref().unwrap() };
        timer.until(self.deadline)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct TimeMeasure(pub i64);

impl TimeMeasure {
    pub const fn from_micros(us: u64) -> Self {
        TimeMeasure(us as i64)
    }

    pub const fn from_mills(ms: u64) -> Self {
        TimeMeasure(ms as i64 * 1000)
    }

    pub const fn from_secs(s: u64) -> Self {
        TimeMeasure(s as i64 * 1000_000)
    }

    pub const fn as_micros(&self) -> i64 {
        self.0 as i64
    }

    pub const fn as_millis(&self) -> i64 {
        self.0 as i64 / 1000
    }

    pub const fn as_secs(&self) -> i64 {
        self.0 as i64 / 1000_000
    }
}

impl Add<isize> for TimeMeasure {
    type Output = Self;
    fn add(self, rhs: isize) -> Self {
        Self(self.0 + rhs as i64)
    }
}

impl Sub<isize> for TimeMeasure {
    type Output = Self;
    fn sub(self, rhs: isize) -> Self {
        Self(self.0 - rhs as i64)
    }
}

#[allow(dead_code)]
pub struct ThreadManager {
    data: u8,
}

impl ThreadManager {
    pub(crate) unsafe fn set_timer(source: Box<dyn TimerSource>) {
        TIMER_SOURCE = Some(source);
    }

    pub(crate) unsafe fn start_threading() {
        // TODO: init threading
        Thread::usleep(1);
    }
}
