# lab1

## 简单总结你实现的功能

在 `TaskControlBlock` 中增加 `task_start_time` 与 `task_syscall_times` 字段。在每个任务首次被转移到 `Running` 状态时记录时间到 `task_start_time`，并在每次产生系统调用时增加 `task_syscall_times` 对应下标的计数。

在调用 `sys_task_info` 时，计算起始时间与当前时间的差值得到 `time`，获取 `task_syscall_times` 数组得到 `syscall_times`，直接设置 `status` 为 `Running`。

## 完成问答题

1. 

`ch2b_bad_address`: 访问非法内存地址导致 PageFault。

```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
```

`ch2b_bad_instructions`: 使用非法指令，被内核杀死。

```
[kernel] IllegalInstruction in application, kernel killed it.
```

`ch2b_bad_register`: 访问非法寄存器，被内核杀死。

```
[kernel] IllegalInstruction in application, kernel killed it.
```

2. 
    1. 此时的 `a0` 寄存器指向内核栈顶，与 `sp` 值相同。
    
    `__restore` 的两种使用情景：
    
    - 开始运行程序
    - Trap 处理完毕后，使用 `__restore` 从内核栈上的 Trap 上下文恢复寄存器
    
    2. 特殊处理了 `sstatus`, `sepc`, `sscratch` 三个寄存器。其中 `sstatus` 的 `SPP` 字段给出 trap 发生前 CPU 所处的特权级信息、`sepc` 记录 trap 发生前最后一条指令的地址、`sscratch` 指向用户栈栈顶。
    
    进入用户态时，CPU 会将当前特权级设置为 `sstatus` 的 `SPP` 字段，跳转到 `sepc` 指向的指令，将栈指针 `sp` 的值与 `sscratch` 互换来恢复用户栈，并继续执行。

    3. 通用寄存器 `x2` `x4` 分别为 `sp` `tp`。`tp` 寄存器除非特殊用途否则不会被用到；`sp` 寄存器目前指向的是内核栈栈顶，其被用来恢复其他寄存器的值，之后会与 `sscratch` 互换来重新获取用户栈栈顶。

    4. 本行之后，`sp` 重新指向用户栈栈顶，`sscratch` 保存进入 trap 之前的状态并指向内核栈栈顶。

    5. 状态切换在 `sret` 指令。该指令将 CPU 当前特权级按照 `sstatus` 的 `SPP` 字段设置为 U 或 S，并跳转到 `sepc` 寄存器所指的指令继续执行。

    6. `__alltraps` 中的 `csrrw` 指令交换 `sp` 与 `sscratch` 的值，让 `sp` 指向内核栈栈顶，而 `sscratch` 指向用户栈栈顶。

    7. 从 U 态进入 S 态是在 trap 发生的瞬间由 CPU 硬件自动完成的。



## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无。

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

无。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
