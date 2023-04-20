// 啥调度也没有，就是一个封装

extern crate alloc;

use alloc::sync::Arc;
use core::ops::Deref;
use crate::BaseManager;
use scheduler::BaseScheduler;
use alloc::vec::Vec;
use spinlock::SpinNoIrq; // TODO: 不确定！！！
//use std::marker::PhantomData;

pub struct NaiveTask<Task, T> {
    inner: Task,
    _marker: Option<T>,
}


impl<Task, T> NaiveTask<Task, T> {
    pub const fn new(inner: Task) -> Self {
        Self {
            inner,
            _marker: None,
        }
    }

    pub const fn inner(&self) -> &Task {
        &self.inner
    }
}

impl<Task, T> const Deref for NaiveTask<Task, T> {
    type Target = Task;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct NaiveManager<T, Task, Scheduler, const SMP: usize> {
    scheduler_collection: Vec<Option<Arc<SpinNoIrq<Scheduler>>>>,
    _markerT: Option<T>,
    _markerTask: Option<Task>,
}

impl<Task, T, Scheduler: BaseScheduler, const SMP: usize> NaiveManager<Task, T, Scheduler, SMP> {
    pub fn new() -> Self {
        let mut tmp_collection: Vec<Option<Arc<SpinNoIrq<Scheduler>>>> = Vec::new();
        for _i in 0..SMP {
            tmp_collection.push(None);
        }
        Self {
            scheduler_collection: tmp_collection,
            _markerT: None,
            _markerTask: None,
        }
    }
}

impl<Task, T, Scheduler: BaseScheduler, const SMP: usize> BaseManager for NaiveManager<Task, T, Scheduler, SMP> 
where
    Scheduler: BaseScheduler<SchedItem = Task>,
{
    type SchedItem = Arc<NaiveTask<Task, T>>;
    type SchedulerItem = Arc<SpinNoIrq<Scheduler>>;
    fn init(&mut self, cpu_id: usize, queue: Self::SchedulerItem) {
        self.scheduler_collection[cpu_id] = Some(queue.clone());
        let mut scheduler = queue.lock();
        scheduler.init();
    }

    fn add_task(&mut self, cpu_id: usize, task: Self::SchedItem) {
        self.scheduler_collection[cpu_id].as_ref().unwrap().lock().add_task(&task.inner)
    }

    fn remove_task(&mut self, cpu_id: usize, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        if let Some(inner) = self.scheduler_collection[cpu_id].as_ref().unwrap().lock().remove_task(&task.inner) {
            Some(Self::SchedItem::new(NaiveTask {
                inner,
                _marker: None,
            }))
        } else {
            None
        }
    }

    fn pick_next_task(&mut self, cpu_id: usize) -> Option<Self::SchedItem> {
        if let Some(inner) = self.scheduler_collection[cpu_id].as_ref().unwrap().lock().pick_next_task() {
            Some(Self::SchedItem::new(NaiveTask {
                inner,
                _marker: None,
            }))
        } else {
            None
        }
    }

    fn put_prev_task(&mut self, cpu_id: usize, prev: Self::SchedItem, _preempt: bool) {
        self.scheduler_collection[cpu_id].as_ref().unwrap().lock().put_prev_task(&prev.inner, _preempt)
    }

    fn task_tick(&mut self, cpu_id: usize, _current: &Self::SchedItem) -> bool {
        self.scheduler_collection[cpu_id].as_ref().unwrap().lock().task_tick(_current)
    }
}
