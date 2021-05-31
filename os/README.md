
#### os/src/mmu

实现物理内存管理。

addr_types.rs：内存管理需要用到的变量类型，如物理页号，虚拟页号，物理地址

kernel_heap.rs：内核堆管理器

page_directory.rs : 实现页表结构并管理SV39三级页表机制以完成对内存的分页管理，

mem_section.rs:实现对内存的分段管理。对于一段连续的、具有相同权限的虚拟地址空间，将其物理页统一集中在`mem_section`内进行管理。对应于ELF文件的Segment。在该文件内还实现了`MemArea`，该类型管理了进程的内存布局

phys_frame_allocator.rs:物理页管理器

#### os/src/syscall

syscall接口

#### os/task

进程管理。实现进程管理器以及进程控制块`TaskControlBlock`
本os为分时多任务操作系统

#### os/src/trap

中断管理。从u态传入的中断由该文件夹下内容进行管理

#### os/src/fs

文件系统。文件描述符，物理文件读写实现于该文件夹。目前仅支持FAT32格式的磁盘

#### /src/console.rs

内核日志宏文件

#### /src/entry.asm

Kernel启动点

#### /src/loader.rs

elf文件加载器

#### /src/main.rs

内核主功能起始点

#### /src/sbi.rs

声明于rustsbi交互的接口

#### /src/timer.rs

定时器

