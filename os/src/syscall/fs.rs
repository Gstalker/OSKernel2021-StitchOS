use crate::mmu::{
    translated_byte_buffer,
    translated_refmut,
    translated_str,
};
use crate::fs::ProgramBuffer;
use crate::task::{current_user_token, current_task, suspend_current_and_run_next};
use crate::sbi::console_getchar;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    match &inner.fd_table.get_mut(fd).cloned() {
        Some(Some(file)) => {
            let buffers = translated_byte_buffer(inner.get_user_token(), buf, len);
            let result = file.lock().write(ProgramBuffer::new(buffers)) as isize;
            return result;
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

use alloc::vec;

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    ERROR!("read_test");
    let task = current_task().unwrap();
    let file_fd = {
        let inner = task.acquire_inner_lock();
        inner.fd_table.get(fd).cloned()
    };
    ERROR!("read_test");
    match file_fd {
        Some(Some(file)) => {
            let t_buf = translated_byte_buffer(
                task.acquire_inner_lock().get_user_token(), 
                buf, 
                len
            );
            unsafe {
                ERROR!("read_test");
                let result  = file.lock().read(ProgramBuffer::new(t_buf)) as isize;
                ERROR!("read_test");
                result
            }
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}