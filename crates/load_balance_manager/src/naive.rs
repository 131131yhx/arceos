// 啥调度也没有，就是一个封装

extern crate alloc;

use alloc::sync::Arc;
use core::ops::Deref;
use crate::BaseManager;
use scheduler::BaseScheduler;
use crate::SimpleRunQueueOperations;
use alloc::vec::Vec;
use spinlock::SpinNoIrq; // TODO: 不确定！！！
//use std::marker::PhantomData;

pub struct NaiveTask<Task, T> {
    inner: Arc<Task>,
    _marker: Option<T>,
}


impl<Task, T> NaiveTask<Task, T> {
    pub const fn new(inner: Arc<Task>) -> Self {
        Self {
            inner,
            _marker: None,
        }
    }

    pub const fn inner(&self) -> &Arc<Task> {
        &self.inner
    }
}

impl<Task, T> const Deref for NaiveTask<Task, T> {
    type Target = Arc<Task>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct NaiveManager<Task, T, const SMP: usize> {
    scheduler_collection: Vec<Option<Arc<SpinNoIrq<dyn SimpleRunQueueOperations<SchedItem = Arc<Task>> + Send + 'static>>>>,
    _markerT: Option<T>,
}

impl<Task, T, const SMP: usize> NaiveManager<Task, T, SMP> {
    pub fn new() -> Self {
        let mut tmp_collection: Vec<Option<Arc<SpinNoIrq<dyn SimpleRunQueueOperations<SchedItem = Arc<Task>> + Send + 'static>>>> = Vec::new();
        for _i in 0..SMP {
            tmp_collection.push(None);
        }
        Self {
            scheduler_collection: tmp_collection,
            _markerT: None,
        }
    }
}

impl<Task, T, const SMP: usize> BaseManager for NaiveManager<Task, T, SMP> {
    type SchedItem = Arc<NaiveTask<Task, T>>;
    type InnerSchedItem = Arc<Task>;
    fn init(&mut self, cpu_id: usize, queue_ref: Arc<SpinNoIrq<dyn SimpleRunQueueOperations<SchedItem = Self::InnerSchedItem> + Send + 'static>>) {
        self.scheduler_collection[cpu_id] = Some(queue_ref.clone());
        let mut scheduler = queue_ref.lock();
        queue_ref.lock().simple_init();
    }

    fn add_task(&mut self, cpu_id: usize, task: Self::SchedItem) {
        self.scheduler_collection[cpu_id].as_ref().unwrap().lock().simple_add_task(&task.inner)
    }

    fn remove_task(&mut self, cpu_id: usize, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        if let Some(inner) = self.scheduler_collection[cpu_id].as_ref().unwrap().lock().simple_remove_task(&task.inner) {
            Some(Self::SchedItem::new(NaiveTask {
                inner,
                _marker: None,
            }))
        } else {
            None
        }
    }

    fn pick_next_task(&mut self, cpu_id: usize) -> Option<Self::SchedItem> {
        if let Some(inner) = self.scheduler_collection[cpu_id].as_ref().unwrap().lock().simple_pick_next_task() {
            Some(Self::SchedItem::new(NaiveTask {
                inner,
                _marker: None,
            }))
        } else {
            None
        }
    }

    fn put_prev_task(&mut self, cpu_id: usize, prev: Self::SchedItem, _preempt: bool) {
        self.scheduler_collection[cpu_id].as_ref().unwrap().lock().simple_put_prev_task(&prev.inner, _preempt)
    }

    fn task_tick(&mut self, cpu_id: usize, _current: &Self::SchedItem) -> bool {
        self.scheduler_collection[cpu_id].as_ref().unwrap().lock().simple_task_tick(_current)
    }
}
