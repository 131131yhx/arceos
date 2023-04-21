#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]

#[macro_use]
extern crate log;

struct KernelGuardIfImpl;

#[crate_interface::impl_interface]
impl kernel_guard::KernelGuardIf for KernelGuardIfImpl {
    fn disable_preempt() {
        #[cfg(all(feature = "multitask", feature = "preempt"))]
        if let Some(curr) = current_may_uninit() {
            curr.disable_preempt();
        }
    }

    fn enable_preempt() {
        #[cfg(all(feature = "multitask", feature = "preempt"))]
        if let Some(curr) = current_may_uninit() {
            curr.enable_preempt(true);
        }
    }
}

struct LogTaskImpl;



#[cfg(not(feature = "std"))]
#[def_interface]
pub trait LogMyTime {
    /// get current time
    fn current_cpu_id() -> Option<usize>;
}

#[crate_interface::impl_interface]
impl LogMyTime for LogTaskImpl {
    fn current_cpu_id() -> Option<usize> {
        #[cfg(feature = "smp")]
        if is_init_ok() {
            Some(axhal::cpu::this_cpu_id())
        } else {
            None
        }
        #[cfg(not(feature = "smp"))]
        Some(0)
    }
}

use crate_interface::{call_interface, def_interface};
pub fn get_current_cpu_id() -> usize {
    call_interface!(LogMyTime::current_cpu_id).unwrap()
}

cfg_if::cfg_if! {
if #[cfg(feature = "multitask")] {

extern crate alloc;

mod run_queue;
mod task;
mod timers;
mod wait_queue;

#[cfg(test)]
mod tests;

use alloc::sync::Arc;

use self::run_queue::{AxRunQueue, RUN_QUEUE};
use self::task::{CurrentTask, TaskInner};

pub use self::task::TaskId;
pub use self::wait_queue::WaitQueue;


cfg_if::cfg_if! {
    if #[cfg(feature = "sched_fifo")] {
        type AxTaskInner = scheduler::FifoTask<TaskInner>;
        type Scheduler = scheduler::FifoScheduler<TaskInner>;
    } else if #[cfg(feature = "sched_rr")] {
        const MAX_TIME_SLICE: usize = 5;
        type AxTaskInner = scheduler::RRTask<TaskInner, MAX_TIME_SLICE>;
        type Scheduler = scheduler::RRScheduler<TaskInner, MAX_TIME_SLICE>;
    } else if #[cfg(feature = "sched_cfs")] {
        type AxTaskInner = scheduler::CFTask<TaskInner>;
        type Scheduler = scheduler::CFScheduler<TaskInner>;
    } else if #[cfg(feature = "sched_sjf")] {
        const alpha_a: usize = 1;
        const alpha_log_b: usize = 4; // 1/16
        type AxTaskInner = scheduler::SJFTask<TaskInner, alpha_a, alpha_log_b>;
        type Scheduler = scheduler::SJFScheduler<TaskInner, alpha_a, alpha_log_b>;
    } else if #[cfg(feature = "sched_mlfq")] {
        const QNUM: usize = 8;
        const BASTTICK: usize = 1;
        const RESETTICK: usize = 100_000;
        type AxTaskInner = scheduler::MLFQTask<TaskInner, QNUM, BASTTICK, RESETTICK>;
        type Scheduler = scheduler::MLFQScheduler<TaskInner, QNUM, BASTTICK, RESETTICK>;
    } else if #[cfg(feature = "sched_rms")] {
        type AxTaskInner = scheduler::RMSTask<TaskInner>;
        type Scheduler = scheduler::RMScheduler<TaskInner>;
    }
}

const SMP : usize = axconfig::SMP;
type AxTask = load_balance_manager::NaiveTask<AxTaskInner, TaskInner>;
type Manager = load_balance_manager::NaiveManager<AxTaskInner, TaskInner, SMP>;

type AxTaskRef = Arc<AxTask>;

pub fn current_may_uninit() -> Option<CurrentTask> {
    CurrentTask::try_get()
}

pub fn current() -> CurrentTask {
    CurrentTask::get()
}

pub fn init_scheduler() {
    info!("Initialize scheduling...");

    self::run_queue::init();
    self::timers::init();

    if cfg!(feature = "sched_fifo") {
        info!("  use FIFO scheduler.");
    } else if cfg!(feature = "sched_rr") {
        info!("  use Round-robin scheduler.");
    } else if cfg!(feature = "sched_cfs") {
        info!("  use CFS.");
    } else if cfg!(feature = "sched_sjf") {
        info!("  use short job first scheduler.");
    } else if cfg!(feature = "sched_mlfq") {
        info!("  use Multi-Level Feedback Queue scheduler.");
    }
}

pub fn init_scheduler_secondary() {
    self::run_queue::init_secondary();
}

/// Handle periodic timer ticks for task manager, e.g. advance scheduler, update timer.
pub fn on_timer_tick() {
    self::timers::check_events();
    RUN_QUEUE[get_current_cpu_id()].lock().scheduler_timer_tick();
}

cfg_if::cfg_if! {

if #[cfg(feature = "sched_cfs")] {
    pub fn spawn<F>(f: F, _nice: isize)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = TaskInner::new(f, "", axconfig::TASK_STACK_SIZE, _nice);
        RUN_QUEUE[get_current_cpu_id()].lock().add_task(task);
    }
} else if #[cfg(feature = "sched_rms")] {
    pub fn spawn<F>(f: F, runtime: usize, period: usize)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = TaskInner::new(f, "", axconfig::TASK_STACK_SIZE, runtime, period);
        RUN_QUEUE[get_current_cpu_id()].lock().add_task(task);
    }
} else {
    pub fn spawn<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = TaskInner::new(f, "", axconfig::TASK_STACK_SIZE);
        RUN_QUEUE[get_current_cpu_id()].lock().add_task(task);
    }
}
}



pub fn yield_now() {
    RUN_QUEUE[get_current_cpu_id()].lock().yield_current();
}

pub fn sleep(dur: core::time::Duration) {
    let deadline = axhal::time::current_time() + dur;
    RUN_QUEUE[get_current_cpu_id()].lock().sleep_until(deadline);
}

pub fn sleep_until(deadline: axhal::time::TimeValue) {
    RUN_QUEUE[get_current_cpu_id()].lock().sleep_until(deadline);
}

pub fn exit(exit_code: i32) -> ! {
    RUN_QUEUE[get_current_cpu_id()].lock().exit_current(exit_code)
}

} else { // if #[cfg(feature = "multitask")]

pub fn yield_now() {}

pub fn exit(exit_code: i32) -> ! {
    debug!("main task exited: exit_code={}", exit_code);
    axhal::misc::terminate()
}

pub fn sleep(dur: core::time::Duration) {
    let deadline = axhal::time::current_time() + dur;
    sleep_until(deadline)
}

pub fn sleep_until(deadline: axhal::time::TimeValue) {
    while axhal::time::current_time() < deadline {
        core::hint::spin_loop();
    }
}

} // else
} // cfg_if::cfg_if!

pub fn run_idle() -> ! {
    loop {
        yield_now();
        debug!("idle task: waiting for IRQs...");
        axhal::arch::wait_for_irqs();
    }
}
