use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::debug;

use crate::loader::{get_app_info, get_num_app, AppInfo};
use crate::mm::{kernel_stack_position, MapPermission, MemorySet, PhysPageNum, VirtAddress, KERNEL_SPACE, TRAP_CONTEXT};
use crate::println;
use crate::sbi::shut_down;
use crate::sync::up::UPSafeCell;
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

use super::context::TaskContext;
use super::switch::__switch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskStatus {
    UnInit,
    READY,
    RUNNING,
    EXITED,
}

struct TaskControlBlock {
    status: TaskStatus,
    cx: TaskContext,
    mem_set: MemorySet,
    trap_ctx_ppn: PhysPageNum,
    base_size: usize,
}

impl TaskControlBlock {
    pub fn new(data: &[u8], app_id:usize)->Self{
        let (mem_set,usp, entry) = MemorySet::new_app_from_elf(data);
        let trap_ctx_ppn = mem_set.page_table.translate(VirtAddress::from(TRAP_CONTEXT).into()).unwrap().ppn();
        let status = TaskStatus::READY;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_frame(kernel_stack_bottom.into(), kernel_stack_top.into(), MapPermission::R|MapPermission::W);
        let block = Self{
            status,
            cx:TaskContext::goto_trap_return(kernel_stack_top),
            mem_set,
            trap_ctx_ppn,
            base_size: usp,
        };
        let trap_ctx = block.get_trap_ctx();
        *trap_ctx = TrapContext::init_new_app(usp, entry, KERNEL_SPACE.exclusive_access().page_table.token(), kernel_stack_top, trap_handler as usize);
        block
    }

    fn get_trap_ctx(&self)->&'static mut TrapContext{
        self.trap_ctx_ppn.get_mut()
    }
}

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current: usize,
}

lazy_static! {
    static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks : Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(
                TaskControlBlock::new(get_app_info(i).mem, i)
            );
        }
        TaskManager {
            num_app,
            inner: unsafe { UPSafeCell::new(TaskManagerInner { tasks, current: 0 }) },
        }
    };
}

impl TaskManager {
    fn mark_current_task_exited(&self) {
        let mut m = self.inner.exclusive_access();
        let current = m.current;
        m.tasks[current].status = TaskStatus::EXITED;
    }
    fn mark_current_task_suspend(&self) {
        let mut m = self.inner.exclusive_access();
        let current = m.current;
        m.tasks[current].status = TaskStatus::READY;
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            debug!("[kernel] scheduling app {}", next);
            let mut m = self.inner.exclusive_access();
            let current = m.current;
            m.current = next;
            m.tasks[next].status = TaskStatus::RUNNING;
            let cur = &mut m.tasks[current].cx as *mut TaskContext;
            let nxt = &mut m.tasks[next].cx as *const TaskContext;
            drop(m);

            unsafe {
                __switch(cur, nxt);
            }
        } else {
            println!("[kernel] all apps exited, will shutdown");
            shut_down(false)
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        let m = self.inner.exclusive_access();
        for i in 1..=self.num_app {
            let idx = (m.current + i) % (self.num_app);
            if m.tasks[idx].status == TaskStatus::READY {
                return Some(idx);
            }
        }
        None
    }

    fn run_first_task(&self) -> ! {
        let mut m = self.inner.exclusive_access();
        let current = 0;
        m.current = current;
        m.tasks[current].status = TaskStatus::RUNNING;
        let nxt = &m.tasks[current].cx;
        let next = nxt as *const TaskContext;
        drop(m);

        debug!("[kernel] scheduling app {}", current);
        let mut unused_buf: TaskContext = TaskContext::zero_init();
        let unused = &mut unused_buf as *mut TaskContext;
        unsafe {
            __switch(unused, next);
        }
        panic!("should shutdown after all apps exited")
    }

    fn get_current_token(&self)->usize{
        let m = self.inner.exclusive_access();
        m.tasks.get(m.current).unwrap().mem_set.page_table.token()
    }

    fn get_current_trap_cx(&self)->&mut TrapContext{
        let m = self.inner.exclusive_access();
        m.tasks.get(m.current).unwrap().get_trap_ctx()
    }
}

pub fn get_current_app() -> AppInfo {
    let cur = TASK_MANAGER.inner.exclusive_access().current;
    get_app_info(cur)
}

pub fn exit_current_task() -> ! {
    mark_current_task_exited();
    run_next_task();
    panic!("should not run here")
}

pub fn suspend_current_task() {
    mark_current_task_suspend();
    run_next_task()
}

fn mark_current_task_exited() {
    TASK_MANAGER.mark_current_task_exited()
}

fn mark_current_task_suspend() {
    TASK_MANAGER.mark_current_task_suspend()
}

pub fn run_next_task() {
    TASK_MANAGER.run_next_task()
}

#[allow(unreachable_code)]
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
    panic!("should not return")
}

pub fn get_current_token()->usize{
    TASK_MANAGER.get_current_token()
}

pub fn get_current_trap_cx()->&'static mut TrapContext{
    TASK_MANAGER.get_current_trap_cx()
}