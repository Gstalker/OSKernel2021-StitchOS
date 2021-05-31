# StitchOS

> for Kendtyre K210
> by csu_Gstalker,csu_Phosphorus15

使用rust语言开发，基于risc-v的小型抢占式os

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
kflash -D ./k210.bin
```

此时，可以使用串口数据读取应用，如screen，MobaXterm等查看板子的运行情况。

你也可以在项目根目录下使用指令`make run`，在固件刷写完毕后立即接入板子。



## 文件结构

虽然目前本系统处于不稳定开发阶段，但目录结构已经本稳定。

本文档将展示本项目的文件结构，以方便后来者继续开发本项目

参阅src.md