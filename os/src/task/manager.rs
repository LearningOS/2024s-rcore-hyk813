//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::config::BIG_STRIDE;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }

    /// impl stride
    pub fn fetch_stride(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut minstride_tcb = self.ready_queue[0].clone();
        let  tt = minstride_tcb.inner_exclusive_access();
        let mut min_stride = tt.stride;
        drop(tt);
        for tcb in &self.ready_queue{
            let task = tcb.inner_exclusive_access();
            if task.stride < min_stride{
                min_stride = task.stride;
                minstride_tcb = tcb.clone(); 
            }
        }
        //if let Some(task) = self.ready_queue.iter().map(f)
        let mut pos = 0;
        for (index,tcb) in self.ready_queue.iter().enumerate(){
            if Arc::ptr_eq(tcb, &minstride_tcb) {
                pos = index;
                break;
            }
        }

        self.ready_queue.remove(pos);
        let mut tt = minstride_tcb.inner_exclusive_access();
        tt.stride  = tt.stride+BIG_STRIDE/tt.priority;
        drop(tt);

        return Some(minstride_tcb);

    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch_stride()
}

// Take a process out of the ready queue with stride algorithm
// pub fn stride_fetch_task()->Option<Arc<TaskControlBlock>> {
//     //trace!("kernel: TaskManager::fetch_task");
//     TASK_MANAGER.exclusive_access().fetch_stride()
// }