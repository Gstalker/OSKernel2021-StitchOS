use alloc::vec::Vec;

mod stdio;
pub mod fat32;
pub mod inode;

pub use stdio::{Stdin, Stdout};

pub struct ProgramBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

pub trait File : Send + Sync {
    fn read(&self, buf: ProgramBuffer) -> usize;
    fn write(&mut self, buf: ProgramBuffer) -> usize;
}

impl ProgramBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}


impl IntoIterator for ProgramBuffer {
    type Item = *mut u8;
    type IntoIter = ProgramBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        ProgramBufferIterator {
            buffers: self.buffers,
            current_buffer: 0,
            current_idx: 0,
        }
    }
}

pub struct ProgramBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}

impl Iterator for ProgramBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_buffer >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.current_buffer][self.current_idx] as *mut _;
            if self.current_idx + 1 == self.buffers[self.current_buffer].len() {
                self.current_idx = 0;
                self.current_buffer += 1;
            } else {
                self.current_idx += 1;
            }
            Some(r)
        }
    }
}