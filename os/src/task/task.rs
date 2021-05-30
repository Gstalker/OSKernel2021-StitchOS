use crate::mmu::{
    MemArea, 
    MemSectionPermission, 
    PhysPageNumber, 
    KERNEL_SPACE, 
    VirtAddr,
    translated_refmut,
};
use super::pid::{
    PidItem,
    pid_alloc,
    KernelStack,
};
use crate::trap::{TrapContext, trap_handler};
use crate::config::{TRAP_CONTEXT, kernel_stack_position};
use super::TaskContext;
use crate::fs::{File, Stdout, Stdin};
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::sync::{Weak, Arc};
use spin::{Mutex, MutexGuard};

pub type FDTable = Vec<Option<Arc<dyn File + Send + Sync>>>;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidItem,
    pub kernel_stack: KernelStack,
    // mutable
    inner: Mutex<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    // trap上下文所在的物理页帧的物理页号
    pub trap_cx_ppn: PhysPageNumber,
    // 应用数据只会出现在小于这个值的位置
    pub base_size: usize,
    // 暂停任务所在内核栈中的位置
    pub task_cx_ptr: usize,
    // 进程状态，enum，见TaskStatus
    pub task_status: TaskStatus,
    // 进程地址空间
    pub mem_area: MemArea,
    // 父进程
    pub parent: Option<Weak<TaskControlBlock>>,
    // 子进程族
    pub children: Vec<Arc<TaskControlBlock>>,
    // 退出编号
    pub exit_code: i32,
    // 进程打开的fd
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
}

impl TaskControlBlockInner {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.mem_area.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // mem_area with elf program headers/trampoline/trap context/user stack
        println!("osc");
        let (mut mem_area, user_sp, entry_point) = MemArea::new_from_elf(elf_data);
        println!("alloc");
        let trap_cx_ppn = mem_area
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
            mem_area,
            trap_cx_ppn,
            base_size: user_sp,
            parent   : None,
            children : Vec::new(),
            exit_code: 0,
            fd_table: vec![
                Some(Arc::new(Stdin)),
                Some(Arc::new(Stdout)),
                Some(Arc::new(Stdout)), // stderr
            ]
        };
        // prepare TrapContext in user space
        println!("prepare trap");
        let trap_cx = task_control_block.get_trap_cx();
        println!("prepare trap start");
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


impl TaskControlBlock {
    pub fn acquire_inner_lock(&self) -> MutexGuard<TaskControlBlockInner> {
        self.inner.lock()
    }
    pub fn new(elf_data: &[u8]) -> Self {
        // mem_area with elf program headers/trampoline/trap context/user stack
        let (mut mem_area, user_sp, entry_point) = MemArea::new_from_elf(elf_data);
        let trap_cx_ppn = mem_area
            .get_phys_frame_by_vpn(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .into();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        // push a task context which goes to trap_return to the top of kernel stack
        let task_cx_ptr = kernel_stack.push(TaskContext::goto_trap_return());
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: Mutex::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: user_sp,
                task_cx_ptr: task_cx_ptr as usize,
                task_status: TaskStatus::Ready,
                mem_area,
                parent: None,
                children: Vec::new(),
                exit_code: 0,
                fd_table: vec![
                    // 0 -> stdin
                    Some(Arc::new(Stdin)),
                    // 1 -> stdout
                    Some(Arc::new(Stdout)),
                    // 2 -> stderr
                    Some(Arc::new(Stdout)),
                ],
            }),
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
    pub fn brk(&self,expend_size : usize) -> Option<isize>{
        let mut inner = self.acquire_inner_lock();
        let mem_area = &mut inner.mem_area;
        mem_area.brk(expend_size)
    }
    pub fn exec(&self, elf_data: &[u8]) {
        // mem_area with elf program headers/trampoline/trap context/user stack
        let (mut mem_area, user_sp, entry_point) = MemArea::new_from_elf(elf_data);
        let trap_cx_ppn = mem_area
            .get_phys_addr_by_va(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .into();
        // **** hold current PCB lock
        let mut inner = self.acquire_inner_lock();
        // substitute mem_area
        inner.mem_area = mem_area;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize trap_cx
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        // **** release current PCB lock
    }
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // ---- hold parent PCB lock
        let mut parent_inner = self.acquire_inner_lock();
        // copy user space(include trap context)
        let mem_area = MemArea::new_from_mem_area(
            &parent_inner.mem_area
        );
        let trap_cx_ppn = mem_area
            .get_phys_frame_by_vpn(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        // push a goto_trap_return task_cx on the top of kernel stack
        let task_cx_ptr = kernel_stack.push(TaskContext::goto_trap_return());
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: Mutex::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: parent_inner.base_size,
                task_cx_ptr: task_cx_ptr as usize,
                task_status: TaskStatus::Ready,
                mem_area,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                exit_code: 0,
                fd_table: new_fd_table,
            }),
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** acquire child PCB lock
        let trap_cx = task_control_block.acquire_inner_lock().get_trap_cx();
        // **** release child PCB lock
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // ---- release parent PCB lock
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn get_ppid(&self) -> Option<isize>{
        if let Some(parent) = &self.acquire_inner_lock().parent{
            Some(parent.upgrade().unwrap().pid.0 as isize)
        }
        else{
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}