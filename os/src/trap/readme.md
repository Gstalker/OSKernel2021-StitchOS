# 可以继续做的改进
目前syscall系统采用的rCore的direct实现。可以改进为vector方法

# 目前存在的问题
kernel是否需要放在虚拟内存里头？

# MMU：分页内存管理
使用RISC-V的SV39功能，提供3级页表内存管理