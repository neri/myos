// Thread Scheduler

use super::executor::Executor;
use super::*;
use crate::arch::cpu::{Cpu, CpuContextData};
use crate::mem::string::*;
use crate::rt::*;
use crate::sync::atomicflags::*;
use crate::sync::semaphore::*;
use crate::sync::spinlock::*;
use crate::system::*;
use crate::window::*;
use crate::*;
use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::*;
use bitflags::*;
use core::cell::UnsafeCell;
use core::fmt::Write;
use core::num::*;
use core::ops::*;
use core::sync::atomic::*;
use core::time::Duration;
use crossbeam_queue::ArrayQueue;

const THRESHOLD_SAVING: usize = 666;
const THRESHOLD_FULL_THROTTLE_MODE: usize = 750;

static mut SCHEDULER: Option<Box<Scheduler>> = None;

static SCHEDULER_ENABLED: AtomicBool = AtomicBool::new(false);

/// System Scheduler
pub struct Scheduler {
    queue_realtime: ThreadQueue,
    queue_higher: ThreadQueue,
    queue_normal: ThreadQueue,
    queue_lower: ThreadQueue,

    locals: Vec<Box<LocalScheduler>>,

    pool: ThreadPool,

    usage: AtomicUsize,
    usage_total: AtomicUsize,
    is_frozen: AtomicBool,
    state: SchedulerState,

    next_timer: Timer,
    sem_timer: Semaphore,
    timer_queue: ArrayQueue<TimerEvent>,
}

#[allow(non_camel_case_types)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SchedulerState {
    Disabled = 0,
    Saving,
    Running,
    FullThrottle,
    MAX,
}

impl Scheduler {
    /// Start scheduler and sleep forever
    pub(crate) fn start(f: fn(usize) -> (), args: usize) -> ! {
        const SIZE_OF_SUB_QUEUE: usize = 64;
        const SIZE_OF_MAIN_QUEUE: usize = 256;

        let queue_realtime = ThreadQueue::with_capacity(SIZE_OF_SUB_QUEUE);
        let queue_higher = ThreadQueue::with_capacity(SIZE_OF_SUB_QUEUE);
        let queue_normal = ThreadQueue::with_capacity(SIZE_OF_MAIN_QUEUE);
        let queue_lower = ThreadQueue::with_capacity(SIZE_OF_SUB_QUEUE);
        let pool = ThreadPool::default();

        let locals = Vec::new();

        unsafe {
            SCHEDULER = Some(Box::new(Self {
                queue_realtime,
                queue_higher,
                queue_normal,
                queue_lower,
                locals,
                pool,
                usage: AtomicUsize::new(0),
                usage_total: AtomicUsize::new(0),
                is_frozen: AtomicBool::new(false),
                state: SchedulerState::Running,
                next_timer: Timer::JUST,
                sem_timer: Semaphore::new(0),
                timer_queue: ArrayQueue::new(100),
            }));
        }

        let sch = Self::shared();
        for index in 0..System::num_of_active_cpus() {
            sch.locals.push(LocalScheduler::new(ProcessorIndex(index)));
        }

        SpawnOption::with_priority(Priority::Realtime).spawn(
            Self::scheduler_thread,
            0,
            "Scheduler",
        );

        SpawnOption::with_priority(Priority::Normal).spawn(f, args, "System");

        SCHEDULER_ENABLED.store(true, Ordering::SeqCst);

        loop {
            unsafe {
                Cpu::halt();
            }
        }
    }

    #[inline]
    #[track_caller]
    fn shared<'a>() -> &'a mut Self {
        unsafe { SCHEDULER.as_mut().unwrap() }
    }

    /// Get the current process if possible
    #[inline]
    pub fn current_pid() -> Option<ProcessId> {
        if Self::is_enabled() {
            Self::current_thread().map(|thread| thread.as_ref().pid)
        } else {
            None
        }
    }

    /// Get the current thread running on the current processor
    #[inline]
    pub fn current_thread() -> Option<ThreadHandle> {
        unsafe {
            Cpu::without_interrupts(|| Self::local_scheduler().map(|sch| sch.current_thread()))
        }
    }

    /// Get the personality instance associated with the current thread
    #[inline]
    pub fn current_personality<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&mut Box<dyn Personality>) -> R,
    {
        Self::current_thread()
            .and_then(|thread| thread.update(|thread| thread.personality.as_mut().map(|v| f(v))))
    }

    /// Perform the preemption
    pub(crate) unsafe fn reschedule() {
        if Self::is_enabled() {
            Cpu::without_interrupts(|| {
                let lsch = Self::local_scheduler().unwrap();
                if lsch.current.as_ref().priority != Priority::Realtime {
                    if lsch.current.update(|current| current.quantum.consume()) {
                        LocalScheduler::switch_context(lsch);
                    }
                }
            });
        }
    }

    pub fn wait_for(object: Option<&SignallingObject>, duration: Duration) {
        unsafe {
            Cpu::without_interrupts(|| {
                let lsch = Self::local_scheduler().unwrap();
                let current = lsch.current;
                if let Some(object) = object {
                    let _ = object.set(current);
                }
                if duration.as_nanos() > 0 {
                    Timer::sleep(duration);
                } else {
                    Scheduler::sleep();
                }
            });
        }
    }

    pub fn sleep() {
        unsafe {
            Cpu::without_interrupts(|| {
                let lsch = Self::local_scheduler().unwrap();
                let current = lsch.current;
                current.as_ref().attribute.insert(ThreadAttributes::ASLEEP);
                LocalScheduler::switch_context(lsch);
            });
        }
    }

    pub fn yield_thread() {
        unsafe {
            Cpu::without_interrupts(|| {
                let lsch = Self::local_scheduler().unwrap();
                LocalScheduler::switch_context(lsch);
            });
        }
    }

    /// Get the scheduler for the current processor
    #[inline]
    fn local_scheduler() -> Option<&'static mut Box<LocalScheduler>> {
        match unsafe { SCHEDULER.as_mut() } {
            Some(sch) => {
                Cpu::current_processor_index().and_then(move |index| sch.locals.get_mut(index.0))
            }
            None => None,
        }
    }

    /// Get the next executable thread from the thread queue
    fn next(index: ProcessorIndex) -> Option<ThreadHandle> {
        let shared = Self::shared();
        if shared.is_frozen.load(Ordering::SeqCst) {
            return None;
        }
        match shared.state {
            SchedulerState::FullThrottle => (),
            SchedulerState::Saving => {
                if index.0 != 0 {
                    return None;
                }
            }
            _ => {
                if System::cpu(index.0).processor_type() != ProcessorCoreType::Main {
                    return None;
                }
            }
        }
        if !shared.next_timer.until() {
            shared.sem_timer.signal();
        }
        if let Some(next) = shared.queue_realtime.dequeue() {
            return Some(next);
        }
        if let Some(next) = shared.queue_higher.dequeue() {
            return Some(next);
        }
        if let Some(next) = shared.queue_normal.dequeue() {
            return Some(next);
        }
        if let Some(next) = shared.queue_lower.dequeue() {
            return Some(next);
        }
        None
    }

    fn enqueue(&mut self, handle: ThreadHandle) {
        match handle.as_ref().priority {
            Priority::Realtime => self.queue_realtime.enqueue(handle).unwrap(),
            Priority::High => self.queue_higher.enqueue(handle).unwrap(),
            Priority::Normal => self.queue_normal.enqueue(handle).unwrap(),
            Priority::Low => self.queue_lower.enqueue(handle).unwrap(),
            _ => unreachable!(),
        }
    }

    /// Retire Thread
    fn retire(thread: ThreadHandle) {
        let handle = thread;
        let shared = Self::shared();
        let thread = handle.as_ref();
        if thread.priority == Priority::Idle {
            return;
        } else if thread.attribute.contains(ThreadAttributes::ZOMBIE) {
            ThreadPool::drop_thread(handle);
        } else if thread.attribute.test_and_clear(ThreadAttributes::AWAKE) {
            thread.attribute.remove(ThreadAttributes::ASLEEP);
            shared.enqueue(handle);
        } else if thread.attribute.contains(ThreadAttributes::ASLEEP) {
            thread.attribute.remove(ThreadAttributes::QUEUED);
        } else {
            shared.enqueue(handle);
        }
    }

    /// Add thread to the queue
    fn add(thread: ThreadHandle) {
        let handle = thread;
        let shared = Self::shared();
        let thread = handle.as_ref();
        if thread.priority == Priority::Idle || thread.attribute.contains(ThreadAttributes::ZOMBIE)
        {
            return;
        }
        if !thread.attribute.test_and_set(ThreadAttributes::QUEUED) {
            if thread.attribute.test_and_clear(ThreadAttributes::AWAKE) {
                thread.attribute.remove(ThreadAttributes::ASLEEP);
            }
            shared.enqueue(handle);
        }
    }

    /// Schedule a timer event
    pub fn schedule_timer(event: TimerEvent) -> Result<(), TimerEvent> {
        let shared = Self::shared();
        shared
            .timer_queue
            .push(event)
            .map(|_| shared.sem_timer.signal())
    }

    /// Scheduler
    fn scheduler_thread(_args: usize) {
        let shared = Self::shared();

        SpawnOption::with_priority(Priority::High).spawn_f(
            Self::statistics_thread,
            0,
            "Statistics",
        );

        let mut events: Vec<TimerEvent> = Vec::with_capacity(100);
        loop {
            if let Some(event) = shared.timer_queue.pop() {
                events.push(event);
                while let Some(event) = shared.timer_queue.pop() {
                    events.push(event);
                }
                events.sort_by(|a, b| a.timer.deadline.cmp(&b.timer.deadline));
            }

            while let Some(event) = events.first() {
                if event.until() {
                    break;
                } else {
                    events.remove(0).fire();
                }
            }

            if let Some(event) = events.first() {
                shared.next_timer = event.timer;
            }
            shared.sem_timer.wait();
        }
    }

    /// Measuring Statistics
    fn statistics_thread(_: usize) {
        let shared = Self::shared();

        let expect = 1_000_000;
        let interval = Duration::from_micros(expect as u64);
        let mut measure = Timer::measure();
        loop {
            Timer::sleep(interval);

            let now = Timer::measure();
            let actual = now.0 - measure.0;
            let actual1000 = actual as usize * 1000;

            let mut usage = 0;
            for thread in shared.pool.data.values() {
                let thread = thread.clone();
                let thread = unsafe { &mut (*thread.get()) };
                let load0 = thread.load0.swap(0, Ordering::SeqCst);
                let load = usize::min(load0 as usize * expect as usize / actual1000, 1000);
                thread.load.store(load as u32, Ordering::SeqCst);
                if thread.priority != Priority::Idle {
                    usage += load;
                }
            }

            let num_cpu = System::num_of_active_cpus();
            let usage_total = usize::min(usage, num_cpu * 1000);
            let usage_per_cpu = usize::min(usage / num_cpu, 1000);
            shared.usage_total.store(usage_total, Ordering::SeqCst);
            shared.usage.store(usage_per_cpu, Ordering::SeqCst);

            if usage_total < THRESHOLD_SAVING {
                shared.state = SchedulerState::Saving;
            } else if usage_total > System::num_of_performance_cpus() * THRESHOLD_FULL_THROTTLE_MODE
            {
                shared.state = SchedulerState::FullThrottle;
            } else {
                shared.state = SchedulerState::Running;
            }

            measure = now;
        }
    }

    #[inline]
    pub fn usage_per_cpu() -> usize {
        let shared = Self::shared();
        shared.usage.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn usage_total() -> usize {
        let shared = Self::shared();
        shared.usage_total.load(Ordering::Relaxed)
    }

    /// Returns the current state of the scheduler.
    pub fn current_state() -> SchedulerState {
        if Self::is_enabled() {
            Self::shared().state
        } else {
            SchedulerState::Disabled
        }
    }

    /// Returns whether or not the thread scheduler is working.
    fn is_enabled() -> bool {
        unsafe { &SCHEDULER }.is_some() && SCHEDULER_ENABLED.load(Ordering::SeqCst)
    }

    /// All threads will stop.
    pub(crate) unsafe fn freeze(force: bool) -> Result<(), ()> {
        let sch = Self::shared();
        sch.is_frozen.store(true, Ordering::SeqCst);
        if force {
            // TODO:
        }
        Ok(())
    }

    fn spawn_f(
        start: ThreadStart,
        args: usize,
        name: &str,
        options: SpawnOption,
    ) -> Option<ThreadHandle> {
        let pid = if options.raise_pid {
            RuntimeEnvironment::raise_pid()
        } else {
            Self::current_pid().unwrap_or(ProcessId(0))
        };
        let thread = RawThread::new(
            pid,
            options.priority,
            name,
            Some(start),
            args,
            options.personality,
        );
        Self::add(thread);
        Some(thread)
    }

    /// Spawning asynchronous tasks
    pub fn spawn_async(task: Task) {
        Self::current_thread().unwrap().update(|thread| {
            if thread.executor.is_none() {
                thread.executor = Some(Executor::new());
            }
            thread.executor.as_mut().unwrap().spawn(task);
        });
    }

    /// Performing Asynchronous Tasks
    pub fn perform_tasks() -> ! {
        Self::current_thread().unwrap().update(|thread| {
            thread.executor.as_mut().map(|v| v.run());
        });
        Self::exit();
    }

    pub fn exit() -> ! {
        Self::current_thread().unwrap().update(|t| t.exit());
        unreachable!()
    }

    pub fn get_idle_statistics(vec: &mut Vec<u32>) {
        let sch = Self::shared();
        vec.clear();
        for thread in sch.pool.data.values() {
            let thread = thread.clone();
            let thread = unsafe { &(*thread.get()) };
            if thread.priority != Priority::Idle {
                break;
            }
            vec.push(thread.load.load(Ordering::Relaxed));
        }
    }

    pub fn print_statistics(sb: &mut StringBuffer, exclude_idle: bool) {
        let sch = Self::shared();
        sb.clear();
        writeln!(sb, "PID PRI %CPU TIME     NAME").unwrap();
        for thread in sch.pool.data.values() {
            let thread = thread.clone();
            let thread = unsafe { &(*thread.get()) };
            if exclude_idle && thread.priority == Priority::Idle {
                continue;
            }

            let load = u32::min(thread.load.load(Ordering::Relaxed), 999);
            let load0 = load % 10;
            let load1 = load / 10;
            write!(
                sb,
                "{:3} {} {} {:2}.{:1}",
                thread.pid.0, thread.priority as usize, thread.attribute, load1, load0,
            )
            .unwrap();

            let time = thread.cpu_time.load(Ordering::Relaxed) / 10_000;
            let dsec = time % 100;
            let sec = time / 100 % 60;
            let min = time / 60_00 % 60;
            let hour = time / 3600_00;
            if hour > 0 {
                write!(sb, " {:02}:{:02}:{:02}", hour, min, sec,).unwrap();
            } else {
                write!(sb, " {:02}:{:02}.{:02}", min, sec, dsec,).unwrap();
            }

            match thread.name() {
                Some(name) => writeln!(sb, " {}", name,).unwrap(),
                None => writeln!(sb, " ({})", thread.handle.as_usize(),).unwrap(),
            }
        }
    }
}

/// Processor Local Scheduler
struct LocalScheduler {
    #[allow(dead_code)]
    index: ProcessorIndex,
    idle: ThreadHandle,
    current: ThreadHandle,
    retired: Option<ThreadHandle>,
}

impl LocalScheduler {
    fn new(index: ProcessorIndex) -> Box<Self> {
        let mut sb = Sb255::new();
        sformat!(sb, "(Idle Core #{})", index.0);
        let idle = RawThread::new(ProcessId(0), Priority::Idle, sb.as_str(), None, 0, None);
        Box::new(Self {
            index,
            idle,
            current: idle,
            retired: None,
        })
    }

    unsafe fn switch_context(lsch: &'static mut Self) {
        Cpu::assert_without_interrupt();

        let current = lsch.current;
        let next = Scheduler::next(lsch.index).unwrap_or(lsch.idle);
        current.update(|thread| {
            let now = Timer::measure().0;
            let diff = now - thread.measure.load(Ordering::SeqCst);
            thread.cpu_time.fetch_add(diff, Ordering::SeqCst);
            thread.load0.fetch_add(diff as u32, Ordering::SeqCst);
            thread.measure.store(now, Ordering::SeqCst);
        });
        if current.as_ref().handle != next.as_ref().handle {
            lsch.retired = Some(current);
            lsch.current = next;

            //-//-//-//-//
            current.update(|current| {
                let next = &next.as_ref().context;
                current.context.switch(next);
            });
            //-//-//-//-//

            let lsch = Scheduler::local_scheduler().unwrap();
            let current = lsch.current;
            current.update(|thread| {
                thread.attribute.remove(ThreadAttributes::AWAKE);
                thread.attribute.remove(ThreadAttributes::ASLEEP);
                thread.measure.store(Timer::measure().0, Ordering::SeqCst);
                // thread.quantum.reset();
            });
            let retired = lsch.retired.unwrap();
            lsch.retired = None;
            Scheduler::retire(retired);
        }
    }

    fn current_thread(&self) -> ThreadHandle {
        self.current
    }
}

#[no_mangle]
pub unsafe extern "C" fn sch_setup_new_thread() {
    let lsch = Scheduler::local_scheduler().unwrap();
    let current = lsch.current;
    current.update(|thread| {
        thread.measure.store(Timer::measure().0, Ordering::SeqCst);
    });
    if let Some(retired) = lsch.retired {
        lsch.retired = None;
        Scheduler::retire(retired);
    }
}

pub struct SpawnOption {
    pub priority: Priority,
    pub raise_pid: bool,
    pub personality: Option<Box<dyn Personality>>,
}

impl SpawnOption {
    #[inline]
    pub const fn new() -> Self {
        Self {
            priority: Priority::Normal,
            raise_pid: false,
            personality: None,
        }
    }

    #[inline]
    pub const fn with_priority(priority: Priority) -> Self {
        Self {
            priority,
            raise_pid: false,
            personality: None,
        }
    }

    #[inline]
    pub fn personality(mut self, personality: Box<dyn Personality>) -> Self {
        self.personality = Some(personality);
        self
    }

    #[inline]
    pub fn spawn_f(self, start: fn(usize), args: usize, name: &str) -> Option<ThreadHandle> {
        Scheduler::spawn_f(start, args, name, self)
    }

    #[inline]
    pub fn spawn(mut self, start: fn(usize), args: usize, name: &str) -> Option<ThreadHandle> {
        self.raise_pid = true;
        Scheduler::spawn_f(start, args, name, self)
    }
}

static mut TIMER_SOURCE: Option<Box<dyn TimerSource>> = None;

pub trait TimerSource {
    /// Create timer object from duration
    fn create(&self, duration: TimeSpec) -> TimeSpec;

    /// Is that a timer before the deadline?
    fn until(&self, deadline: TimeSpec) -> bool;

    fn measure(&self) -> TimeSpec;

    fn from_duration(&self, val: Duration) -> TimeSpec;
    fn to_duration(&self, val: TimeSpec) -> Duration;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Timer {
    deadline: TimeSpec,
}

impl Timer {
    pub const JUST: Timer = Timer {
        deadline: TimeSpec(0),
    };

    #[inline]
    pub fn new(duration: Duration) -> Self {
        let timer = unsafe { TIMER_SOURCE.as_ref().unwrap() };
        Timer {
            deadline: timer.create(duration.into()),
        }
    }

    #[inline]
    pub const fn is_just(&self) -> bool {
        self.deadline.0 == 0
    }

    #[inline]
    pub fn until(&self) -> bool {
        if self.is_just() {
            false
        } else {
            let timer = unsafe { TIMER_SOURCE.as_ref().unwrap() };
            timer.until(self.deadline)
        }
    }

    #[inline]
    pub(crate) unsafe fn set_timer(source: Box<dyn TimerSource>) {
        TIMER_SOURCE = Some(source);
    }

    #[track_caller]
    pub fn sleep(duration: Duration) {
        if Scheduler::is_enabled() {
            let timer = Timer::new(duration);
            let mut event = TimerEvent::one_shot(timer);
            while timer.until() {
                match Scheduler::schedule_timer(event) {
                    Ok(()) => {
                        Scheduler::sleep();
                        return;
                    }
                    Err(e) => {
                        event = e;
                        Scheduler::yield_thread();
                    }
                }
            }
        } else {
            panic!("Scheduler unavailable");
        }
    }

    #[inline]
    pub fn usleep(us: u64) {
        Self::sleep(Duration::from_micros(us));
    }

    #[inline]
    pub fn msleep(ms: u64) {
        Self::sleep(Duration::from_millis(ms));
    }

    #[inline]
    pub fn measure() -> TimeSpec {
        unsafe { TIMER_SOURCE.as_ref() }.unwrap().measure()
    }

    #[inline]
    pub fn monotonic() -> Duration {
        Self::measure().into()
    }

    #[inline]
    fn timespec_to_duration(val: TimeSpec) -> Duration {
        unsafe { TIMER_SOURCE.as_ref() }.unwrap().to_duration(val)
    }

    #[inline]
    fn duration_to_timespec(val: Duration) -> TimeSpec {
        unsafe { TIMER_SOURCE.as_ref() }.unwrap().from_duration(val)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSpec(pub usize);

impl Add<TimeSpec> for TimeSpec {
    type Output = Self;
    #[inline]
    fn add(self, rhs: TimeSpec) -> Self::Output {
        TimeSpec(self.0 + rhs.0)
    }
}

impl From<TimeSpec> for Duration {
    #[inline]
    fn from(val: TimeSpec) -> Duration {
        Timer::timespec_to_duration(val)
    }
}

impl From<Duration> for TimeSpec {
    #[inline]
    fn from(val: Duration) -> TimeSpec {
        Timer::duration_to_timespec(val)
    }
}

pub struct TimerEvent {
    timer: Timer,
    timer_type: TimerType,
}

#[derive(Debug, Copy, Clone)]
pub enum TimerType {
    OneShot(ThreadHandle),
    Window(WindowHandle, usize),
}

#[allow(dead_code)]
impl TimerEvent {
    pub fn one_shot(timer: Timer) -> Self {
        Self {
            timer,
            timer_type: TimerType::OneShot(Scheduler::current_thread().unwrap()),
        }
    }

    pub fn window(window: WindowHandle, timer_id: usize, timer: Timer) -> Self {
        Self {
            timer,
            timer_type: TimerType::Window(window, timer_id),
        }
    }

    pub fn until(&self) -> bool {
        self.timer.until()
    }

    pub fn fire(self) {
        match self.timer_type {
            TimerType::OneShot(thread) => thread.wake(),
            TimerType::Window(window, timer_id) => {
                window.post(WindowMessage::Timer(timer_id)).unwrap()
            }
        }
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum Priority {
    Idle = 0,
    Low,
    Normal,
    High,
    Realtime,
}

impl Priority {
    pub fn is_useful(self) -> bool {
        match self {
            Priority::Idle => false,
            _ => true,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Quantum {
    current: u8,
    default: u8,
}

impl Quantum {
    const fn new(value: u8) -> Self {
        Quantum {
            current: value,
            default: value,
        }
    }

    #[allow(dead_code)]
    fn reset(&mut self) {
        self.current = self.default;
    }

    fn consume(&mut self) -> bool {
        if self.current > 1 {
            self.current -= 1;
            false
        } else {
            self.current = self.default;
            true
        }
    }
}

impl From<Priority> for Quantum {
    fn from(priority: Priority) -> Self {
        match priority {
            Priority::High => Quantum::new(25),
            Priority::Normal => Quantum::new(10),
            Priority::Low => Quantum::new(5),
            _ => Quantum::new(1),
        }
    }
}

#[derive(Default)]
struct ThreadPool {
    data: BTreeMap<ThreadHandle, Arc<UnsafeCell<Box<RawThread>>>>,
    lock: Spinlock,
}

impl ThreadPool {
    #[inline]
    #[track_caller]
    fn synchronized<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        unsafe {
            Cpu::without_interrupts(|| {
                let shared = Self::shared();
                shared.lock.synchronized(f)
            })
        }
    }

    #[inline]
    #[track_caller]
    fn shared<'a>() -> &'a mut Self {
        &mut Scheduler::shared().pool
    }

    fn add(thread: Box<RawThread>) {
        Self::synchronized(|| {
            let shared = Self::shared();
            let handle = thread.handle;
            shared
                .data
                .insert(handle, Arc::new(UnsafeCell::new(thread)));
        });
    }

    fn drop_thread(handle: ThreadHandle) {
        Self::synchronized(|| {
            let shared = Self::shared();
            shared.data.remove(&handle);
        });
    }

    fn get<'a>(&self, key: &ThreadHandle) -> Option<&'a Box<RawThread>> {
        Self::synchronized(|| self.data.get(key).map(|v| v.clone().get()))
            .map(|thread| unsafe { &(*thread) })
    }

    fn get_mut<F, R>(&mut self, key: &ThreadHandle, f: F) -> Option<R>
    where
        F: FnOnce(&mut RawThread) -> R,
    {
        let thread = Self::synchronized(move || self.data.get_mut(key).map(|v| v.clone()));
        thread.map(|thread| unsafe {
            let thread = thread.get();
            f(&mut *thread)
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct ThreadHandle(NonZeroUsize);

impl ThreadHandle {
    #[inline]
    fn new(val: usize) -> Option<Self> {
        NonZeroUsize::new(val).map(|x| Self(x))
    }

    /// Acquire the next thread ID
    #[inline]
    fn next() -> ThreadHandle {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        ThreadHandle::new(NEXT_ID.fetch_add(1, Ordering::Relaxed)).unwrap()
    }

    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0.get()
    }

    #[inline]
    #[track_caller]
    fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut RawThread) -> R,
    {
        let shared = ThreadPool::shared();
        shared.get_mut(self, f).unwrap()
    }

    #[inline]
    fn get<'a>(&self) -> Option<&'a Box<RawThread>> {
        let shared = ThreadPool::shared();
        shared.get(self)
    }

    #[inline]
    #[track_caller]
    fn as_ref<'a>(&self) -> &'a RawThread {
        self.get().unwrap()
    }

    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.get().and_then(|v| v.name())
    }

    #[inline]
    fn wake(&self) {
        self.as_ref().attribute.insert(ThreadAttributes::AWAKE);
        Scheduler::add(*self);
    }

    #[inline]
    pub fn join(&self) -> usize {
        self.get().map(|t| t.sem.wait());
        0
    }
}

const THREAD_NAME_LENGTH: usize = 32;

type ThreadStart = fn(usize) -> ();

#[allow(dead_code)]
struct RawThread {
    /// Architectural context data
    context: CpuContextData,
    stack: Option<Box<[u8]>>,

    /// IDs
    pid: ProcessId,
    handle: ThreadHandle,

    // Properties
    sem: Semaphore,
    personality: Option<Box<dyn Personality>>,
    attribute: AtomicBitflags<ThreadAttributes>,
    priority: Priority,
    quantum: Quantum,

    // Statistics
    measure: AtomicUsize,
    cpu_time: AtomicUsize,
    load0: AtomicU32,
    load: AtomicU32,

    // Executor
    executor: Option<Executor>,

    /// Thread Name
    name: [u8; THREAD_NAME_LENGTH],
}

bitflags! {
    struct ThreadAttributes: usize {
        const QUEUED    = 0b0000_0000_0000_0001;
        const ASLEEP    = 0b0000_0000_0000_0010;
        const AWAKE     = 0b0000_0000_0000_0100;
        const ZOMBIE    = 0b0000_0000_0000_1000;
    }
}

impl Into<usize> for ThreadAttributes {
    fn into(self) -> usize {
        self.bits()
    }
}

impl AtomicBitflags<ThreadAttributes> {
    fn to_char(&self) -> char {
        if self.contains(ThreadAttributes::ZOMBIE) {
            'Z'
        } else if self.contains(ThreadAttributes::AWAKE) {
            'W'
        } else if self.contains(ThreadAttributes::ASLEEP) {
            'S'
        } else if self.contains(ThreadAttributes::QUEUED) {
            'R'
        } else {
            '-'
        }
    }
}

use core::fmt;
impl fmt::Display for AtomicBitflags<ThreadAttributes> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[allow(dead_code)]
impl RawThread {
    fn new(
        pid: ProcessId,
        priority: Priority,
        name: &str,
        start: Option<ThreadStart>,
        arg: usize,
        personality: Option<Box<dyn Personality>>,
    ) -> ThreadHandle {
        let handle = ThreadHandle::next();

        let mut name_array = [0; THREAD_NAME_LENGTH];
        Self::set_name_array(&mut name_array, name);

        let mut thread = Self {
            context: CpuContextData::new(),
            stack: None,
            pid,
            handle,
            sem: Semaphore::new(0),
            attribute: AtomicBitflags::empty(),
            priority,
            quantum: Quantum::from(priority),
            measure: AtomicUsize::new(0),
            cpu_time: AtomicUsize::new(0),
            load0: AtomicU32::new(0),
            load: AtomicU32::new(0),
            executor: None,
            personality,
            name: name_array,
        };
        if let Some(start) = start {
            unsafe {
                let size_of_stack = CpuContextData::SIZE_OF_STACK;
                let mut stack = Vec::with_capacity(size_of_stack);
                stack.resize(size_of_stack, 0);
                let stack = stack.into_boxed_slice();
                thread.stack = Some(stack);
                let stack = thread.stack.as_mut().unwrap().as_mut_ptr() as *mut c_void;
                thread
                    .context
                    .init(stack.add(size_of_stack), start as usize, arg);
            }
        }
        ThreadPool::add(Box::new(thread));
        handle
    }

    fn exit(&mut self) -> ! {
        self.sem.signal();
        self.personality.as_mut().map(|v| v.on_exit());
        self.personality = None;

        // TODO:
        Timer::sleep(Duration::from_secs(2));
        self.attribute.insert(ThreadAttributes::ZOMBIE);
        Scheduler::sleep();
        unreachable!();
    }

    fn set_name_array(array: &mut [u8; THREAD_NAME_LENGTH], name: &str) {
        let mut i = 1;
        for c in name.bytes() {
            if i >= THREAD_NAME_LENGTH {
                break;
            }
            array[i] = c;
            i += 1;
        }
        array[0] = i as u8 - 1;
    }

    fn set_name(&mut self, name: &str) {
        RawThread::set_name_array(&mut self.name, name);
    }

    fn name<'a>(&self) -> Option<&'a str> {
        let len = self.name[0] as usize;
        match len {
            0 => None,
            _ => core::str::from_utf8(unsafe { core::slice::from_raw_parts(&self.name[1], len) })
                .ok(),
        }
    }
}

#[derive(Debug)]
pub struct SignallingObject(AtomicUsize);

impl SignallingObject {
    const NONE: usize = 0;

    pub fn new() -> Self {
        Self(AtomicUsize::new(
            Scheduler::current_thread().unwrap().as_usize(),
        ))
    }

    pub fn set(&self, value: ThreadHandle) -> Result<(), ()> {
        let value = value.as_usize();
        match self
            .0
            .compare_exchange(Self::NONE, value, Ordering::SeqCst, Ordering::Relaxed)
        {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    pub fn load(&self) -> Option<ThreadHandle> {
        ThreadHandle::new(self.0.load(Ordering::SeqCst))
    }

    pub fn unbox(&self) -> Option<ThreadHandle> {
        ThreadHandle::new(self.0.swap(Self::NONE, Ordering::SeqCst))
    }

    pub fn wait(&self, duration: Duration) {
        Scheduler::wait_for(Some(self), duration)
    }

    pub fn signal(&self) {
        if let Some(thread) = self.unbox() {
            thread.wake()
        }
    }
}

impl From<usize> for SignallingObject {
    fn from(value: usize) -> Self {
        Self(AtomicUsize::new(value))
    }
}

impl From<SignallingObject> for usize {
    fn from(value: SignallingObject) -> usize {
        value.0.load(Ordering::Acquire)
    }
}

struct ThreadQueue(ArrayQueue<NonZeroUsize>);

impl ThreadQueue {
    fn with_capacity(capacity: usize) -> Self {
        Self(ArrayQueue::new(capacity))
    }
    fn dequeue(&self) -> Option<ThreadHandle> {
        self.0.pop().map(|v| ThreadHandle(v))
    }
    fn enqueue(&self, data: ThreadHandle) -> Result<(), ()> {
        self.0.push(data.0).map_err(|_| ())
    }
}
