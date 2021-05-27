pub struct Stdin;

pub struct Stdout;

use super::{File, ProgramBuffer};
use crate::sbi::{console_getchar, console_putchar};
use crate::task::suspend_current_and_run_next;

impl File for Stdin {
    fn read(&self, mut user_buf: ProgramBuffer) -> usize {
        assert_eq!(user_buf.len(), 1);
        let mut c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_current_and_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe { user_buf.buffers[0].as_mut_ptr().write_volatile(ch); }
        1
    }

    fn write(&self, _user_buf: ProgramBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn read(&self, _user_buf: ProgramBuffer) -> usize{
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: ProgramBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
}