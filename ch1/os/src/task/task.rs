use lazy_static::lazy_static;

use crate::println;
use crate::sync::up::UPSafeCell;
use crate::loader::{get_num_app, AppInfo, MAX_APP_NUM, get_app_info, init_app_cx};

use super::context::TaskContext;
use super::switch::__switch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskStatus {
    UnInit,
    READY,
    RUNNING,
    EXITED,
}

#[derive(Clone, Copy)]
struct TaskControlBlock{
    status: TaskStatus,
    cx: TaskContext,
}

pub struct TaskManager{
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner{
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current: usize,
}

lazy_static!{
    static ref TASK_MANAGER:TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock{
                status:TaskStatus::UnInit,
                cx: TaskContext::zero_init(),
            };
            MAX_APP_NUM
        ];
        for i in 0..num_app{
            tasks[i].cx = TaskContext::goto_restore(init_app_cx(i));
            tasks[i].status = TaskStatus::READY;
        }
        TaskManager{
            num_app,
            inner:unsafe {
                UPSafeCell::new(TaskManagerInner { tasks, current: 0})
            }
        }
    };
}

impl TaskManager {
    fn mark_current_task_exited(&self){
        let mut m = self.inner.exclusive_access();
        let current = m.current;
        m.tasks[current].status = TaskStatus::EXITED;
    }
    fn mark_current_task_suspend(&self){
        let mut m = self.inner.exclusive_access();
        let current = m.current;
        m.tasks[current].status = TaskStatus::READY;
    }

    fn run_next_task(&self){
        if let Some(next) = self.find_next_task() {
            let mut m = self.inner.exclusive_access();
            let current = m.current;
            m.current = next;
            m.tasks[next].status = TaskStatus::RUNNING;
            let cur = &mut m.tasks[current].cx as *mut TaskContext;
            let nxt = &mut m.tasks[next].cx as *const TaskContext;
            drop(m);
            unsafe {
                __switch(cur,nxt)
            }
        }else{
            panic!("all application exited")
        }
    }

    fn find_next_task(&self)->Option<usize>{
        let m = self.inner.exclusive_access();
        for i in 1..=self.num_app{
            let idx = (m.current+i)%(self.num_app);
            if m.tasks[idx].status == TaskStatus::READY{
                return Some(idx)
            }
        }
        println!("line -1: {}", line!());
        None
    }

    fn run_first_task(&self)->!{
        let mut m = self.inner.exclusive_access();
        let current = 0;
        m.current = current;
        m.tasks[current].status = TaskStatus::RUNNING;
        let nxt=&m.tasks[current].cx;
        let next = nxt as *const TaskContext;

        //todo debug
        println!("next exec is ra:{:X}, ksp:{:X}",nxt.ra, nxt.sp);

        drop(m);

        let mut unused_buf: TaskContext=TaskContext::zero_init();
        let unused = &mut unused_buf as *mut TaskContext;
        unsafe {
            __switch(unused, next)
        }
    }

}

pub fn get_current_app()->AppInfo{
    let cur = TASK_MANAGER.inner.exclusive_access().current;
    get_app_info(cur)
}

pub fn exit_current_task()->!{
    mark_current_task_exited();
    run_next_task();
    panic!("should not run here")
}

pub fn suspend_current_task(){
    mark_current_task_suspend();
    run_next_task()
}


fn mark_current_task_exited(){
    TASK_MANAGER.mark_current_task_exited()
}

fn mark_current_task_suspend(){
    TASK_MANAGER.mark_current_task_suspend()
}

pub fn run_next_task(){
    TASK_MANAGER.run_next_task()
}

#[allow(unreachable_code)]
pub fn run_first_task(){
    TASK_MANAGER.run_first_task();
    panic!("should not return")
}