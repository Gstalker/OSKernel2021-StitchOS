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
        const CREATE = 0x40;
        const DIR = 0x0200000;
    }
}
use kfat32::file::WriteType;

impl Into<WriteType> for OpenFlags {
    fn into(self) -> WriteType {
        WriteType::OverWritten
    }
}

use crate::fs::fat32;

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let mut task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    let path = translated_str(inner.get_user_token(), path);

    let dir = inner.work_dir.clone();
    let dir_str = alloc::str::from_utf8(dir.as_slice()).unwrap();
    let cwd = fat32::fat32_path(dir_str, None).get_inode().unwrap();

    let next_id = inner.alloc_fd();
    let flags = OpenFlags::from_bits_truncate(flags);
    let file = fat32::fat32_path(&path, Some(cwd.clone()));

    let dir = flags.contains(OpenFlags::DIR);

    if !file.exists() {
        if flags.contains(OpenFlags::CREATE) {
            if flags.contains(OpenFlags::DIR) {
                file.mkdirs().unwrap();
            } else {
                fat32::create_file(&cwd, &fat32::fat32_root_dir(), &path, true).unwrap();
            }
        }
    }

    use alloc::sync::Arc;
    use spin::Mutex;

    let new_file : Arc<Mutex<dyn crate::fs::File + Send + Sync>> = {
        if dir {
            Arc::new(Mutex::new(fat32::SysDir()))
        } else {
            Arc::new(Mutex::new(fat32::SysFile::new(file.open().unwrap(), flags)))
        }
    };

    inner.fd_table[next_id] = Some(new_file);
    next_id as isize
}


pub fn sys_dup(fd: usize) -> isize {
    let mut task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    match inner.fd_table.get(fd).cloned().flatten() {
        Some(fd) => { 
            let id = inner.alloc_fd();
            inner.fd_table[id] = Some(fd);
            id as isize
        }
        None => -1
    }
}

pub fn sys_dup3(fd: usize, neo: usize, _flags: usize) -> isize {
    let mut task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if inner.fd_table.len() > fd || inner.fd_table[fd].is_none() {
        return -1
    }
    match inner.fd_table.get(neo) {
        Some(Some(_)) => -1,
        Some(None) => {
            inner.fd_table[neo] = inner.fd_table[fd].clone();
            neo as isize
        }
        _ => {
            let len = inner.fd_table.len();
            inner.fd_table.reserve(neo - len);
            while inner.fd_table.len() < neo {
                inner.fd_table.push(None)
            };
            let fd = inner.fd_table[fd].clone();
            inner.fd_table.push(fd);
            neo as isize
        }
    }

}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}