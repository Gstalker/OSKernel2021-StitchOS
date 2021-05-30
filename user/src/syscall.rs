const SYSCALL_GETCWD : usize = 17;
const SYSCALL_DUP    :usize = 23;
const SYS_MOUNT : usize = 40;
const SYS_CHIDR : usize = 49;
const SYSCALL_OPEN: usize = 56;
const SYS_CLOSE : usize = 57;
const SYS_PIPE2 : usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYS_FSTATE   : usize = 80;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_UNAME: usize = 160;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_GETPPID: usize = 173;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_BRK : usize = 214;
const SYSCALL_MMAP : usize = 222;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_MKDIR : usize = 1030;
#[repr(C)]
#[derive(Debug)]
pub struct utsname{
    pub sysname : [u8;65],
    pub nodename : [u8;65],
    pub release : [u8;65],
    pub version : [u8;65],
    pub machine : [u8;65],
    pub domainname : [u8;65],
}



fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), "{x17}" (id)
            : "memory"
            : "volatile"
        );
    }
    ret
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_mut_ptr() as usize, buffer.len()])
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_getppid() -> isize {
    syscall(SYSCALL_GETPPID,[0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

// pub fn sys_pipe(pipe: &mut [usize]) -> isize {
//     syscall(SYSCALL_PIPE, [pipe.as_mut_ptr() as usize, 0, 0])
// }


pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn sys_brk(expend_size : usize) -> isize {
    syscall(SYSCALL_BRK, [expend_size,0,0])
}

pub fn sys_uname(us : *mut utsname) -> isize{
    syscall(SYSCALL_UNAME, [us as usize,0,0])
}