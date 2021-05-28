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
    fn write(&self, buf: ProgramBuffer) -> usize;
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