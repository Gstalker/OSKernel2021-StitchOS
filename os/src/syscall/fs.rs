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

            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            
            file.lock().write(ProgramBuffer::new(buffers)) as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

use alloc::vec;

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    match inner.fd_table.get(fd) {
        Some(Some(file)) => {
            unsafe {
                file.lock().read(ProgramBuffer::new(vec![
                    core::slice::from_raw_parts_mut(buf, len)
                ])) as isize
            }
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}