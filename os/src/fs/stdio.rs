pub struct Stdin;

pub struct Stdout;

use super::{File, ProgramBuffer};
use crate::sbi::{console_getchar, console_putchar};
use crate::task::suspend_current_and_run_next;

impl File for Stdin {
    //if c == 0 ,return -2
    fn read(&self, mut user_buf: ProgramBuffer) -> usize {
        let len = user_buf.len();
        let mut i = 0;
        let mut c: usize;
        while i < len {
            c = console_getchar();
            if c == 0 {
                suspend_current_and_run_next();
            } else {
                let ch = c as u8;
                unsafe { user_buf.buffers[i].as_mut_ptr().write_volatile(ch); }
                i += 1;
            }
        }
        len
    }

    fn write(&mut self, _user_buf: ProgramBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn read(&self, _user_buf: ProgramBuffer) -> usize{
        panic!("Cannot read from stdout!");
    }
    fn write(&mut self, user_buf: ProgramBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
}