### ch4练习

#### 一、重写sys_get_time  sys_get_task_info

##### sys_get_time  

```Rust
/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let buffers =
        translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let mut time_val_ptr = &time_val as *const _ as *const u8;
    for buffer in buffers {
        unsafe {
            time_val_ptr.copy_to(buffer.as_mut_ptr(), buffer.len());
            time_val_ptr = time_val_ptr.add(buffer.len());
        }
    }
    0
}
```

ts是一个地址了，要在该地址上存储timeval的值。涉及到地址也就涉及到本章的地址空间问题。我们就正常获取到timeval的值，然后通过虚拟地址写入到真实的物理地址中去。

#####  sys_get_task_info

原本的做法是获取当前task的tcb。但是现在tcb加了一些别的东西，不支持clone trait了。但是我们也就只需要里面的syscall_times数组和start_time。所以获取这两个的clone就可以了。然后依葫芦画瓢写入。

```Rust
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
```

#### 二、完成sys_mmap

- syscall ID：222

- 申请长度为 len 字节的物理内存（不要求实际物理内存位置，可以随便找一块），将其映射到 start 开始的虚存，内存页属性为 prot

- - 参数：

    start 需要映射的虚存起始地址，要求按页对齐len 映射字节长度，可以为 0prot：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0

- 返回值：执行成功则返回 0，错误返回 -1

- - 说明：

    为了简单，目标虚存区间要求按页对齐，len 可直接按页向上取整，不考虑分配失败时的页回收。

- - 可能的错误：

    start 没有按页大小对齐prot & !0x7 != 0 (prot 其余位必须为0)prot & 0x7 = 0 (这样的内存无意义)[start, start + len) 中存在已经被映射的页物理内存不足

根据提示，我们先处理能根据start、len、port就能返回-1的情况

```rust
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    //println!("-----------------[mmap_in_syscall]----------------");
    //let align_len = (len+4095)/4096;
    if (start%4096!=0 )|| (port & !0x7 != 0) || (port & 0x7 == 0) { 
        return -1;
    } 
    return current_task_mmap(start, len, port);
}
```

然后我们分析一下mmap的实现链条。文档信息量太大，一下子不好理清楚，但是我们可以知道：我们已经有了虚拟地址的开始和结束地址。然后我们申请这一段虚拟内存加入到当前task的memory_set中->也就是创建一个memory_area加入到memory_set的Vec中。那么接下来就到memory_set的定义中找一下相关函数，追踪一下会发生什么。

创建一个memory_area需要给出虚拟地址的开始和结束地址、映射方式、prot，和data。通过memory_area的new函数可以自动构建出来。我们这里data肯定是None。

memory_set的定义中有个insert_frame的方法。查看它的push函数，需要把这段内存空间整理（map）到memory_area的页表上，再看如何map。

这个map函数中，对vpnrange中的每个vpn都调用map_one函数。即处理一段内存空间跨越多个页表的情况。

在map_one函数中：我们alloc了一个页，获得了这个页的ppn和它的frameTracker，然后我们需要把这个vpn和frameTracker加入到我们memory_area的data_frame中。然后计算pte_flags，加入到页表中。

加入页表的map函数中，根据vpn查找了当前memory_set的页表！！因此这里我们将会判断这段空间是否已经被映射！

```rust
        let pte = self.find_pte_create(vpn).unwrap();
        //assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        if pte.is_valid() {
            println!("vpn {:?} is mapped before mapping", vpn);
            return -1;
        }
```

这里出现问题就可以返回了-1了。

总结一下：syscall->为了方便转到TCB中->memory_set 's insert_frame->new memory_area and push -> 处理好set的页表（copy好数据然后push） -> 处理每一个virtualPage的页表 -> 处理单个页表：分配frame，map入set的页表->检查标志位（是否重用等其他问题），然后创建新的页表项。mmap完成。

我们从最后一步递归返回结果到系统调用处即可。

#### 三、完成sys_munmap

刚刚因为memory_set中定义了insert_frame函数，所以我们自上而下分析成功了。现在没有remove_area函数了。所以我们要自己实现。

通过上一步的insert的链条我们可以整理出要做这些事：

找到当前memory_set -> 在对应的memory_area中找出对应的vpn并unmap掉分配的内存 ->根据vpn，在set的页表中unmap掉。

刚好我们发现了memory_area的unmap和pageTable的unmap都已经实现了。那么关键就在于找出有哪些memory_area包括了这一段我们要unmap的区间。

如何判断是否包含呢，通过已有的mmap我们可以确信，虚拟内存都是一段一段申请的，所以如果某个area的vpnrange中完全包括了st和ed所在的vpn，那么就是包含。我们调用area的unmap函数就好了。结果照样一步一步返回上来。



