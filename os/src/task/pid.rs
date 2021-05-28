use alloc::vec::Vec;
use lazy_static::*;
use spin::Mutex;
use crate::mmu::{
    MemSectionPermission,
    KERNEL_SPACE,
    VirtAddr,
    VirtPageNumber,
};
use crate::config::{
    TRAMPOLINE,
    PAGE_SIZE,
    KERNEL_STACK_SIZE,
};

pub struct PidItem(pub usize);


struct PidAllocator{
    current : usize,
    recycle_stack : Vec<usize>,
}

impl PidAllocator{
    fn new() -> Self{
        PidAllocator{
            current : 0,
            recycle_stack : Vec::new(),
        }
    }
    fn alloc(&mut self) -> PidItem{
        let result;
        if self.recycle_stack.is_empty() != true{
            result = self.recycle_stack.pop().unwrap()
        }
        else{
            result = self.current;
            self.current += 1;
        }
        PidItem(result)
    }
    fn dealloc(&mut self,pid : usize){
        self.recycle_stack.push(pid);
    }
}

lazy_static!{
    static ref PID_ALLOCATOR : Mutex<PidAllocator> = Mutex::new(PidAllocator::new());
}

pub fn pid_alloc() -> PidItem{
    PID_ALLOCATOR.lock().alloc()
}

impl Drop for PidItem {
    fn drop(&mut self) {
        PID_ALLOCATOR.lock().dealloc(self.0)
    }
}

pub struct KernelStack{
    pid : usize,
}

//return the end and top poisition of A kernel stack which marked by pid_alloc
pub fn get_kernel_stack_poision(pid : usize) -> (usize,usize) {
    let top = TRAMPOLINE - pid*(KERNEL_STACK_SIZE + PAGE_SIZE);
    let end = top - KERNEL_STACK_SIZE;
    (end,top)
}

impl KernelStack{
    pub fn get_position(&self) -> (usize,usize){
        get_kernel_stack_poision(self.pid)
    }
    pub fn get_top(&self) -> usize{
        let (a,b) = self.get_position();
        b
    }
    pub fn get_bottom(&self) -> usize{
        let (a,b) = self.get_position();
        a
    }
    pub fn new(pid_item : &PidItem) -> Self{
        let (end,top) = get_kernel_stack_poision(pid_item.0);
        KERNEL_SPACE
            .lock()
            .add_framed_section(
                end.into(),
                top.into(),
                MemSectionPermission::R | MemSectionPermission::W,
                None,
            );
        Self{
            pid : pid_item.0
        }
    }
    pub fn push<T>(&self,value:T) -> *mut T where T : Sized,{
        let top = self.get_top();
        let ptr = (top - core::mem::size_of::<T>()) as *mut T;
        unsafe{ *ptr = value; }
        ptr
    }
}

impl Drop for KernelStack{
    fn drop(&mut self){
        let end = self.get_bottom();
        let end_va : VirtAddr = end.into();
        KERNEL_SPACE
            .lock()
            .remove_framed_section_by_start_vpn(end_va.into());
    }
}