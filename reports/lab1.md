# chapter3练习

```
fn sys_task_info(ti: *mut TaskInfo) -> isize
```

```
struct TaskInfo {
    status: TaskStatus,
    syscall_times: [u32; MAX_SYSCALL_NUM],
    time: usize
}
```

- - 参数：

    ti: 待查询任务信息

- 返回值：执行成功返回0，错误返回-1

- - 说明：

    相关结构已在框架中给出，只需添加逻辑实现功能需求即可。在我们的实验中，系统调用号一定小于 500，所以直接使用一个长为 `MAX_SYSCALL_NUM=500` 的数组做桶计数。运行时间 time 返回系统调用时刻距离任务第一次被调度时刻的时长，也就是说这个时长可能包含该任务被其他任务抢占后的等待重新调度的时间。由于查询的是当前任务的状态，因此 TaskStatus 一定是 Running。（助教起初想设计根据任务 id 查询，但是既不好定义任务 id 也不好写测例，遂放弃 QAQ）调用 `sys_task_info` 也会对本次调用计数。

- - 提示：

    大胆修改已有框架！除了配置文件，你几乎可以随意修改已有框架的内容。程序运行时间可以通过调用 `get_time()` 获取，注意任务运行总时长的单位是 ms。系统调用次数可以考虑在进入内核态系统调用异常处理函数之后，进入具体系统调用函数之前维护。阅读 TaskManager 的实现，思考如何维护内核控制块信息（可以在控制块可变部分加入需要的信息）。虽然系统调用接口采用桶计数，但是内核采用相同的方法进行维护会遇到什么问题？是不是可以用其他结构计数？

### 1.实现能够找到status

实验框架是，我们有一个全局的TaskManager来管理任务的运行，可以知道当前运行的任务是哪个，以及所有TCB。

直接在TCB中多一个成员记录status即可。然后在系统调用的时候获得当前TCB的信息，即可获得status。

### 2.实现syscall_times数组的维护

同理，TCB中加一个syscall_times数组。在每次系统调用的时候，对当前syscall_id加一

这里选择在系统调用前，也就是在trap_handler分发和处理trap的时候调用一个函数update_syscall_times(id)，来实现。

这个函数选择在task/mod.rs中实现

```rust
pub fn update_syscall_times(id:usize){
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].syscall_times[id]+=1;
}
```

```rust
/// trap handler
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
                               // trace!("into {:?}", scause.cause());
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            cx.sepc += 4;
            // get system call return value
            update_syscall_times(cx.x[17]);
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
```

### 3.维护调用时间

TCB中加一个start_time，在系统调用处理的时候get当前时间，一减即可。不过好像start_time都是0

维护start_time：run_first_task的时候初始化，run_next_task的时候初始化。

### 4.实现系统调用

```rust
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let current_tcb = get_clone_of_current_tcb();
    let start_time = current_tcb.start_time;
    let increment_time = get_time_ms()-start_time;
    unsafe {
        *_ti = TaskInfo{
        status:TaskStatus::Running,
        syscall_times:current_tcb.syscall_times,
        time:increment_time,
        } 
    }
    0
}
```

获得当前TCB，处于不要出现所有权问题以及思路简单，获得当前TCB的一个clone拿来用。然后赋值即可。

```rust
/// Because a syscall has been called,So the app's array:syscall_times,should be updated.
pub fn update_syscall_times(id:usize){
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].syscall_times[id]+=1;
}

```

