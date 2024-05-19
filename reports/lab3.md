### ch5练习

#### 一、通过之前的测例

属实是有点麻了，改的地方不算少。

上来就踩了一个坑，使用make test CHAPTER=$ID之后，会修改makefile文件。然后自己本地就测不了了。要自己本次测试就

cd os 

make run TEST=1 

然后输入应用名手动测。

sys_get_time 还是非常简单，一模一样。

taskinfo那个就有点麻烦了。之前我们只有一个应用在运行。现在是多个进程。我们的在对ch4的task的定义和思考要挪到processer上。

PROCESSOR包裹一层TCB的Arc，TCB里面包裹一层TCB_inner

注意到take_current，current函数，所以我们通过PROCESSOR的current获取当前的任务，也就是ch4的TASKMANAGER

```rust

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
```

然后各种移植，有点烦ww。

#### 二、spwan

说是不等于fork+exec。但是反正是新开一个进程，装载新任务。

总之先fork一下、然后装载

```rust
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    
    
    let token = current_user_token();
    let path = translated_str(token, path);
        if let Some(data) = get_app_data_by_name(path.as_str()) {
        new_task.exec(data);
        let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
        trap_cx.x[10] = 0;
        add_task(new_task);
        return new_pid as isize;
    }
```

#### 三、stride调度

系统调用直接赋值。

要找到调度的地方，把下一个任务调到queue的最前面。那这个调度的地方一定能管理所有未运行的进程，即TASKMANAGER

直接在调度的地方写就行了。这种应用类的还是比较简单。

学到了Arc::ptr_eq()函数。