# StitchOS

> for Kendtyre K210
> by csu_Gstalker,csu_Phosphorus15

使用rust语言开发，基于risc-v的分时多任务系统

## 环境配置

### Linux

安装rust开发环境，并配置riscv交叉编译环境

```shell
curl https://sh.rustup.rs -sSf | sh
rustup install nightly
rustup default nightly
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils --vers ~0.2
rustup component add llvm-tools-preview
rustup component add rust-src
```

## 编译方法

首先将板子的UART口连接到主机上。然后执行下列指令

```shell
git clone https://github.com/Gstalker/OScomp-Kernel.git
cd OScomp-kernel
make all
```

上述指令会在项目根目录下生成一个名为k210.bin的文件。该文件即为系统固件，使用[kflash.py](https://github.com/kendryte/kflash.py)刷入开发板即可。

```shell
python3 -m pip install kflash
kflash -D dan ./k210.bin
```

此时，可以使用串口数据读取应用，如screen，MobaXterm等查看板子的运行情况。

你也可以在项目根目录下使用指令`make run`，在固件刷写完毕后立即接入板子。



## 文件结构

虽然目前本系统处于不稳定开发阶段，但目录结构已经本稳定。

本文档将展示本项目的文件结构，以方便后来者继续开发本项目

参阅src.md



## 内核功能进度记录

### 内存管理

内核态内存管理已全部完成。

- 逻辑分段式进程内存空间管理 （见memsection.rs）
- 基于SV39机制的物理页管理（见pagedirectory.rs）
- ELF LOADER

### trap与syscall接口

已完成上下文切换和syscall调用接口。

### 任务调度

分时多任务系统的任务管理基本完成

- 基于TaskControlBlock(TCB)的进程管理
- 基于时间片和中断机制的进程执行调度
- 内核栈，pid，fd表等进程信息
- 暂停，运行，僵死三大进程状态

### 文件系统

- 完成FAT32磁盘驱动程序
- 完成文件描述符，并重载STDIN和STDOUT

### 系统调用

已完成的系统调用见见下述代码

```rust
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_READ => sys_read(args[0], args[1] as *mut u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SYSCALL_BRK => sys_brk(args[0]),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_GETPPID => sys_getppid(),
        SYSCALL_UNAME => sys_uname(args[0] as *mut utsname),
        SYSCALL_DUP => sys_dup(args[0]),
        SYSCALL_DUP3 => sys_dup3(args[0], args[1], args[2]),
        SYSCALL_OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_GETCWD => getcwd(args[0] as *mut u8, args[1] as u32),
        SYSCALL_CHDIR => chdir(args[0] as *const u8),
        SYSCALL_MKDIR => sys_mkdir(args[0] as *const u8, args[1] as u32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}

```



## 未来工作计划

### 进程间通信

基于文件描述符的PIPE系统调用

### shell内核化

将shell程序连接进内核，以应对没有sdcard情况下的系统运行

### 完善文件系统

目前文件系统处于不稳定状态，需要进一步完善以配合用户态程序

### 批量测试syscall

syscall需要更多的测试用例以证明其健壮性