

use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};

use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        //println!("--------------in sys_mutex_create,got mutex_id={}--------------",id);
        if process_inner.deadlock_test {
            process_inner.available[id]+=1;
            process_inner.work[id]+=1;
        }


        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        let id =process_inner.mutex_list.len() as isize - 1;
        
        //println!("--------------in sys_mutex_create,got mutex_id={}--------------",id);
        if process_inner.deadlock_test {
            process_inner.available[id as usize]+=1;
            process_inner.work[id as usize]+=1;
        }

        return id;
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());

    // let tid = current_task().unwrap()
    //     .inner_exclusive_access()
    //     .res
    //     .as_ref()
    //     .unwrap()
    //     .tid;
    //println!("--------------in sys_mutex_unlock,got mutex_id={},tid={}--------------",mutex_id,tid);   

    if process_inner.deadlock_test{
        if process_inner.work[mutex_id]<=0{
            return -0xDEAD;
        }
        process_inner.work[mutex_id]-=1;
    }

    drop(process_inner);
    drop(process);
    mutex.lock();
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());

    // let tid = current_task().unwrap()
    //     .inner_exclusive_access()
    //     .res
    //     .as_ref()
    //     .unwrap()
    //     .tid;
    //println!("--------------in sys_mutex_unlock,got mutex_id={},tid={}--------------",mutex_id,tid);    
    if process_inner.deadlock_test{
        process_inner.work[mutex_id]+=1;
    }


    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    // let tid = current_task().unwrap()
    // .inner_exclusive_access()
    // .res
    // .as_ref()
    // .unwrap()
    // .tid;

    


    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));

        //println!("--------------in sys_sem_down,got sem_id={},tid={}--------------",id,tid); 

        if process_inner.deadlock_test{
            process_inner.available[id]+=res_count;
            process_inner.work[id]+=res_count;
        }

        id
    } else { 
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        let id = process_inner.semaphore_list.len() - 1;

        //println!("--------------in sys_sem_down,got sem_id={},tid={}--------------",id,tid); 

        if process_inner.deadlock_test{
            process_inner.available[id]+=res_count;
            process_inner.work[id]+=res_count;
        }

        return id as isize;
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());

    let tid = current_task().unwrap()
    .inner_exclusive_access()
    .res
    .as_ref()
    .unwrap()
    .tid;

    //println!("--------------in sys_sem_up,got sem_id={},tid={}--------------",sem_id,tid); 
    if process_inner.deadlock_test {
        process_inner.work[sem_id]+=1;
        process_inner.allocation[tid][sem_id]-=1;
    }

    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());

    let tid = current_task().unwrap()
    .inner_exclusive_access()
    .res
    .as_ref()
    .unwrap()
    .tid;

    //println!("--------------in sys_sem_down,got sem_id={},tid={}--------------",sem_id,tid); 
    if process_inner.deadlock_test{
        process_inner.need[tid][sem_id]+=1;

        let num_thread_plus=3;
        let sem_plus=4;
        for i in 0..num_thread_plus{
            process_inner.finished[i]=false;
        }
        loop{
            let mut change = false;
            for i in 0..num_thread_plus{
                if process_inner.finished[i]==true{
                    continue;
                }
                let mut release = true;
                for j in 0..sem_plus{
                    if process_inner.work[j]<process_inner.need[i][j]{
                        release = false; 
                        break;
                    }
                }
                if release{
                    change = true;
                    for j in 0..sem_plus{
                        process_inner.work[j]+=process_inner.allocation[i][j];
                    }
                    process_inner.finished[i] = true;
                    //println!("--------release sem,tid={}----------",i);
                }
            }
            //println!("--------no release----------");
            if change==false{
                break;
            }
        }
        for i in 0..num_thread_plus{
            if process_inner.finished[i] != true{
                //println!("--------find dead,tid={}----------",tid);
                return -0xDEAD;
            }
        }
        process_inner.need[tid][sem_id]-=1;
        process_inner.work[sem_id]-=1;
        process_inner.allocation[tid][sem_id]+=1;
    }

    drop(process_inner);
    sem.down();
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    if enabled==0||enabled==1{
        let process=current_process();
        let mut inner = process.inner_exclusive_access();
        inner.deadlock_test = enabled == 1;
        return  0;
    }
    -1
}
