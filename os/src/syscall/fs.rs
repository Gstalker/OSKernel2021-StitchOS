const FD_STDIN : usize = 0;
const FD_STDOUT: usize = 1;
const FD_STDERR: usize = 2;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize,buf: *const u8, len: usize) -> isize{
    match fd{
        FD_STDIN =>{
            return 0 as iszise;
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}