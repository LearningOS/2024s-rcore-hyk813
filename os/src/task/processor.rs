//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::config::MAX_SYSCALL_NUM;
use crate::fs::{Stat, StatMode};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

/// get info in PROCESSOR
pub fn get_current_processor_info() -> ([u32; MAX_SYSCALL_NUM], usize,TaskStatus){
    let binding = PROCESSOR.exclusive_access().current().unwrap();
    let inner = binding.inner_exclusive_access();
    let start_time = inner.start_time;
    let syscall_times = inner.syscall_times;
    let status = inner.task_status;
    println!("Start_time:{0},status{1}",start_time,status as usize);

    return (syscall_times,start_time,status);
}


///update sys_call_times
pub fn update_syscall_times(id:usize){
    let binding = PROCESSOR.exclusive_access().current().unwrap();
    let mut inner = binding.inner_exclusive_access();
    inner.syscall_times[id]+=1;
}

/// finish mmap in current_processor_addressspace
pub fn current_processor_mmap(start: usize, len: usize, port: usize) -> isize{
    let binding = PROCESSOR.exclusive_access().current().unwrap();
    let mut inner = binding.inner_exclusive_access();
    //println!("--------------[mmap_to_current_tcb]---------------");
    inner.mmap_tcb(start, len, port)
}

/// finish unmap in current_processor_addressspace
pub fn current_processor_munmap(start: usize, len: usize) ->isize{
    let binding = PROCESSOR.exclusive_access().current().unwrap();
    let mut inner = binding.inner_exclusive_access();
    inner.mumap_tcb(start, len)
}


/// get stat by fd
pub fn current_processor_fstat(fd: usize)->Option<Stat>{
    let task = current_task().unwrap();
    let  inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len(){
        return None;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        let mut  stat =Stat::new(0, StatMode::NULL, 0);
        file.get_stat(&mut stat);
        return Some(stat);
    }
    None
}