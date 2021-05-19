use crate::mmu::{
    MemArea, 
    MemSectionPermission, 
    PhysPageNumber, 
    KERNEL_SPACE, 
    VirtAddr,
};
use crate::trap::{TrapContext, trap_handler};
use crate::config::{TRAP_CONTEXT, kernel_stack_position};
use super::TaskContext;

pub struct TaskControlBlock {
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub memory_set: MemArea,
    pub trap_cx_ppn: PhysPageNumber,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        
        let (mut memory_set, user_sp, entry_point) = MemArea::new_from_elf(elf_data);
        
        let trap_cx_ppn = memory_set
            .get_phys_frame_by_vpn(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap();
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE
            .lock() 
            .add_framed_section(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
                MemSectionPermission::R | MemSectionPermission::W,
                None
            );
        let task_cx_ptr = (kernel_stack_top - core::mem::size_of::<TaskContext>()) as *mut TaskContext;
        unsafe { *task_cx_ptr = TaskContext::goto_trap_return(); }
        let task_control_block = Self {
            task_cx_ptr: task_cx_ptr as usize,
            task_status,
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}