#![feature(associated_type_defaults)]
#![no_std]
pub mod block_dev;
pub mod bpb;
pub mod volume;
pub mod tool;
pub mod dir;
pub mod entry;
pub mod file;
pub mod fat;

const BUFFER_SIZE: usize = 512;
