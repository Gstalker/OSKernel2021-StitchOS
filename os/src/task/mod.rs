mod context;
mod switch;
mod task;
mod controller;
mod processor;
mod pid;

use crate::loader::{
    get_app_data_by_name,
    get_app_data_by_name_oj,
};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;
use alloc::vec::Vec;
use controller::pop_task;
use lazy_static::*;

pub use context::TaskContext;
pub use processor::{
    run_tasks,
    current_task,
    current_user_token,
    current_trap_cx,
    take_current_task,
    schedule,
};
pub use controller::add_task;
pub use pid::{PidItem, pid_alloc, KernelStack};

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- hold current PCB lock
    let mut task_inner = task.acquire_inner_lock();
    let task_cx_ptr2 = task_inner.get_task_cx_ptr2();
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current PCB lock

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr2);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();
    // **** hold current PCB lock
    let mut inner = task.acquire_inner_lock();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ hold initproc PCB lock here
    {
        let mut initproc_inner = INITPROC.acquire_inner_lock();
        for child in inner.children.iter() {
            child.acquire_inner_lock().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB lock here

    inner.children.clear();
    // deallocate user space
    inner.mem_area.recycle_data_pages();
    drop(inner);
    // **** release current PCB lock
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let _unused: usize = 0;
    schedule(&_unused as *const _);
}

lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = {
        let app_data = get_app_data_by_name_oj("initproc").unwrap();
        DEBUG!("HELLO?");
        let app_path = Vec::from(['/' as u8]);
        Arc::new(
            TaskControlBlock::new(app_data,app_path)
        )
    };
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
    DEBUG!("HELLO?");
}

pub fn run_oj(){
    add_task_oj("brk");
    add_task_oj("chdir");
    add_task_oj("clone");
    add_task_oj("close");
    add_task_oj("dup");
    add_task_oj("dup2");
    add_task_oj("execve");
    add_task_oj("exit");
    add_task_oj("fork");
    add_task_oj("fstat");
    add_task_oj("getcwd");
    add_task_oj("getdents");
    add_task_oj("getpid");
    add_task_oj("getppid");
    add_task_oj("gettimeofday");
    add_task_oj("mkdir");
    add_task_oj("mmap");
    add_task_oj("mount");
    add_task_oj("munmap");
    add_task_oj("open");
    add_task_oj("openat");
    add_task_oj("pipe");
    add_task_oj("read");
    add_task_oj("sleep");
    add_task_oj("times");
    add_task_oj("umount");
    add_task_oj("uname");
    add_task_oj("unlink");
    add_task_oj("wait");
    add_task_oj("waitpid");
    add_task_oj("write");
    add_task_oj("yield");
}

#[allow(unused)]
pub fn add_task_oj(path : &str){
    if let Some(app_data) = get_app_data_by_name_oj(path){
        DEBUG!("HELLO?");
        let app_path = Vec::from(['/' as u8]);
        let task = Arc::new(
            TaskControlBlock::new(app_data,app_path)
        );
        add_task(task);
    }
    else{
        ERROR!("Couldn't add tasl : {}",path);
    }
    
}