const SYSCALL_GETCWD : usize = 17;
const SYSCALL_DUP    :usize = 23;
const SYS_MOUNT : usize = 40;
const SYS_CHIDR : usize = 49;
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

mod fs;
mod process;

use fs::*;
use process::*;

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
        SYSCALL_BRK =>  sys_brk(args[0]),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_GETPPID => sys_getppid(),
        SYSCALL_UNAME => sys_uname(args[0] as *mut utsname),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
