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
    let task = current_task().unwrap();
    let file_fd = {
        let inner = task.acquire_inner_lock();
        inner.fd_table.get(fd).cloned()
    };
    match file_fd {
        Some(Some(file)) => {
            let t_buf = translated_byte_buffer(
                task.acquire_inner_lock().get_user_token(), 
                buf, 
                len
            );
            file.lock().read(ProgramBuffer::new(t_buf)) as isize
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}
use kfat32::file::WriteType;

impl Into<WriteType> for OpenFlags {
    fn into(self) -> WriteType {
        WriteType::OverWritten
    }
}

use crate::fs::fat32;

pub fn sys_open(path: &str, flags: u32) -> isize {
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    let next_id = inner.fd_table.len();
    let flags = OpenFlags::from_bits(flags);
    fat32::fat32_path(path, None);
    -1
}
