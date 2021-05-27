use crate::mmu::translated_byte_buffer;
use crate::task::{current_user_token, current_task_fd};
use crate::fs::ProgramBuffer;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let fd_table = current_task_fd().unwrap();
    match fd_table.get(fd) {
        Some(file_opt) => {
            if let Some(file) = file_opt {
                let buffers = translated_byte_buffer(current_user_token(), buf, len);
                file.write(ProgramBuffer::new(buffers));
                len as isize
            } else {
                -1
            }
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}