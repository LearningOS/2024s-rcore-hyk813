//! Process management syscalls

use core::{intrinsics::size_of};

use crate::{
    config::MAX_SYSCALL_NUM, mm::translated_byte_buffer, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_clone_of_info_in_tcb, suspend_current_and_run_next, TaskStatus,current_task_mmap,
        current_task_munmap,
    }, timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let  insert_buffers =  translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>());
    let mut time_val_ptr =&time_val as *const _ as *const u8; 
    for buffer in insert_buffers{
        unsafe{
            time_val_ptr.copy_to(buffer.as_mut_ptr(), buffer.len());
            time_val_ptr=time_val_ptr.add(buffer.len());
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let info = get_clone_of_info_in_tcb();
    let task_info =TaskInfo{
        status:info.2,
        syscall_times:info.0,
        time:get_time_ms(),
    };
    let  insert_buffers =  translated_byte_buffer(current_user_token(), _ti as *const u8, size_of::<TaskInfo>());
    let mut taskinfo_ptr =&task_info as *const _ as *const u8; 
    for buffer in insert_buffers{
        unsafe{
            taskinfo_ptr.copy_to(buffer.as_mut_ptr(), buffer.len());
            taskinfo_ptr = taskinfo_ptr.add(buffer.len());
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    //println!("-----------------[mmap_in_syscall]----------------");
    //let align_len = (len+4095)/4096;
    if (start%4096!=0 )|| (port & !0x7 != 0) || (port & 0x7 == 0) { 
        return -1;
    } 
    return current_task_mmap(start, len, port);
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");

    if start%4096!=0 || len %4096!=0{
        return -1;
    }
    return current_task_munmap(start,len);


}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
