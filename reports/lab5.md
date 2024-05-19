### ch8练习

#### 一、enable_deadlock_detect

current_task的概念被current_process替换。所以放在原来我们在TCB里设置我们想要的东西，现在我们在PCB里面设置死锁开关

btw，调度相关的数据结构也设置在其中。

#### 二、死锁检测算法

目前的 mutex 和 semaphore 相关的系统调用不会分析资源的依赖情况，用户程序可能出现死锁。 我们希望在系统中加入死锁检测机制，当发现可能发生死锁时拒绝对应的资源获取请求。 一种检测死锁的算法如下：

定义如下三个数据结构：

- 可利用资源向量 Available ：含有 m 个元素的一维数组，每个元素代表可利用的某一类资源的数目， 其初值是该类资源的全部可用数目，其值随该类资源的分配和回收而动态地改变。 Available[j] = k，表示第 j 类资源的可用数量为 k。
- 分配矩阵 Allocation：n * m 矩阵，表示每类资源已分配给每个线程的资源数。 Allocation[i,j] = g，则表示线程 i 当前己分得第 j 类资源的数量为 g。
- 需求矩阵 Need：n * m 的矩阵，表示每个线程还需要的各类资源数量。 Need[i,j] = d，则表示线程 i 还需要第 j 类资源的数量为 d 。

算法运行过程如下：

1. 设置两个向量: 工作向量 Work，表示操作系统可提供给线程继续运行所需的各类资源数目，它含有 m 个元素。初始时，Work = Available ；结束向量 Finish，表示系统是否有足够的资源分配给线程， 使之运行完成。初始时 Finish[0..n-1] = false，表示所有线程都没结束；当有足够资源分配给线程时， 设置 Finish[i] = true。
2. 从线程集合中找到一个能满足下述条件的线程

```
1Finish[i] == false;
2Need[i,j] ≤ Work[j];
```

若找到，执行步骤 3，否则执行步骤 4。

1. 当线程 thr[i] 获得资源后，可顺利执行，直至完成，并释放出分配给它的资源，故应执行:

```
1Work[j] = Work[j] + Allocation[i, j];
2Finish[i] = true;
```

跳转回步骤2

1. 如果 Finish[0..n-1] 都为 true，则表示系统处于安全状态；否则表示系统处于不安全状态，即出现死锁。

出于兼容性和灵活性考虑，我们允许进程按需开启或关闭死锁检测功能。为此我们将实现一个新的系统调用： `sys_enable_deadlock_detect` 。

##### 实现



1.先实现系统调用。在PCB里面加一个变量控制。然后用PROCESSOR全局实例获取(给出代码已经封装)然后这个变量赋为enabled。

2.布置死锁算法实现所需要的向量。选择了无脑的方式，初始化了5个

1. available：
2. allocation：
3. need：
4. work：
5. finished：

观察测例。所谓死锁检测，就是能检测出mutex和sem使用的非法情况。那么检测算法中的向量根据信号量的值、是否获取互斥锁决定。所以感觉是个非常非常简化的判断。也不需要用到全部向量。

定义：

mutex：create的时候第id个available加1 work=available。unlock的时候，第id个work++ 。lock的时候第id个work--。

sem：create的时候，需要把第id个available加res_count个，work=available。

up的时候，第id个work++，tid的第id个alloc--。

down的时候。tid的第id个need++。检测死锁。没有的话就第i个work--，tid的第id个alloc++，然后need--。有的话直接返回