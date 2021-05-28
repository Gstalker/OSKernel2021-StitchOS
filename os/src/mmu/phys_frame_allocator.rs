use super::addr_types::{PhysPageNumber,PhysAddr};
use alloc::vec::Vec;
use crate::config::{PAGE_SIZE,MEMORY_END};
use spin::Mutex;
use lazy_static::*;
use core::fmt::{self, Debug, Formatter};
///mod crate::config;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNumber>;
    fn dealloc(&mut self,ppn: PhysPageNumber);
}

// 使用栈管理未分配的内存页
pub struct PhysFrameAllocator{
    head : usize,
    tail : usize,
    recycle_stack : Vec<usize>
}

impl FrameAllocator for PhysFrameAllocator{
    fn new() -> Self{ 
        Self{
            head : 0,
            tail : 0,
            recycle_stack : Vec::new()
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNumber>{
        //如果栈中有未分配的物理页，优先分配
        if let Some(ppn) = self.recycle_stack.pop(){
            Some(ppn.into())
        }
        //若栈中没有物理页且物理内存耗尽，返回None
        else if self.head >= self.tail{
            None
        }
        //分配物理内存
        else{
            self.head += PAGE_SIZE;
            Some((self.head - PAGE_SIZE).into())
        }
    }
    fn dealloc(&mut self, p: PhysPageNumber){
        let ppn : usize = p.into();
        //合法检测：ppn是否在栈中或者ppn是处于未被分配的区域
        if ppn > self.head || self.recycle_stack
        .iter()
        .find(|&v| {*v == ppn})
        .is_some(){
            panic!("Error!PhysPage {:X} has not been allocated!",ppn);
        }
        self.recycle_stack.push(ppn);
    }
}

impl PhysFrameAllocator {
    fn init(&mut self,h: usize,t: usize){ 
        self.head = h;
        self.tail = t;
        self.recycle_stack = Vec::new();
    }
}

type FrameAllocatorImpl = PhysFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocatorImpl> 
        = Mutex::new(FrameAllocatorImpl::new());
}

pub fn init_frame_allocator(){
    extern "C"{ 
        fn ekernel();
    }
    FRAME_ALLOCATOR.lock().init(
        PhysAddr::from(ekernel as usize).celi(),
        PhysAddr::from(MEMORY_END).floor()
    );
}

pub struct FrameItem{
    pub ppn : PhysPageNumber,
}

impl FrameItem{
    pub fn new(ppn : PhysPageNumber) -> Self{ 
        let bytes = ppn.get_bytes_array();
        for i in bytes{
            *i = 0;
        }
        Self{
            ppn 
        }
    }
}

impl Drop for FrameItem{
    fn drop(&mut self){
        phys_frame_dealloc(self.ppn);
    }
}

impl Debug for FrameItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameItem:PPN={:#x}", self.ppn.0))
    }
}

pub fn phys_frame_alloc() -> Option<FrameItem> {
    FRAME_ALLOCATOR
        .lock()
        .alloc()
        .map(|ppn| FrameItem::new(ppn))
}

fn phys_frame_dealloc(ppn:PhysPageNumber){
    FRAME_ALLOCATOR
        .lock()
        .dealloc(ppn)
}