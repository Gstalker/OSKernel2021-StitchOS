use super::TaskControlBlock;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::*;


pub struct TaskController{
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl TaskController{
    pub fn new() -> Self{
        Self{
            ready_queue: VecDeque::new(),
        }
    }
    pub fn push(&mut self,task : Arc<TaskControlBlock>){
        self.ready_queue.push_back(task);
    }
    pub fn pop(&mut self) -> Option<Arc<TaskControlBlock>>{
        self.ready_queue.pop_front()
    }
}

lazy_static!{
    pub static ref TASK_CONTROLLER : Mutex<TaskController> = Mutex::new(TaskController::new());
}

pub fn add_task(task : Arc<TaskControlBlock>){
    TASK_CONTROLLER.lock().push(task)
}

pub fn pop_task() -> Option<Arc<TaskControlBlock>>{
    TASK_CONTROLLER.lock().pop()
}