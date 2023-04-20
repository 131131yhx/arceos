#![cfg_attr(not(test), no_std)]
#![feature(const_trait_impl)]
#![feature(const_mut_refs)]

mod naive;

pub use naive::{NaiveManager, NaiveTask};

pub trait BaseManager {
    type SchedItem;
    type SchedulerItem;
    // 需要逐个 Scheduler 进行 init
    fn init(&mut self, cpu_id: usize, queue_ref: Self::SchedulerItem);
    // 注意：默认是对所有调度器都初始化后，才会进行操作。
    // 下面全是对已有任务的封装，包含原有调度器的操作以及现有调度器的操作
    fn add_task(&mut self, cpu_id: usize, task: Self::SchedItem);
    fn remove_task(&mut self, cpu_id: usize, task: &Self::SchedItem) -> Option<Self::SchedItem>;
    fn pick_next_task(&mut self, cpu_id: usize) -> Option<Self::SchedItem>;
    fn put_prev_task(&mut self, cpu_id: usize, prev: Self::SchedItem, preempt: bool);
    fn task_tick(&mut self, cpu_id: usize, current: &Self::SchedItem) -> bool;
}
