use alloc::sync::*;
use alloc::vec::*;
use alloc::vec;
use spin::Mutex;

use super::fat32::Inode;

pub struct OSInode {
    readable: bool,
    writable: bool,
    pub inner: Mutex<OSInodeInner>,
}

pub struct OSInodeInner {
    offset: usize,
    pub inode: Arc<Inode>,
}

impl OSInode {
    pub fn new(
        readable: bool,
        writable: bool,
        inode: Arc<Inode>,
    ) -> Self {
        Self {
            readable,
            writable,
            inner: Mutex::new(OSInodeInner {
                offset: 0,
                inode,
            }),
        }
    }

}