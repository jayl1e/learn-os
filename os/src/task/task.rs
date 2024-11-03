use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use alloc::{collections, task};
use alloc::string::{String, ToString};
use lazy_static::lazy_static;
use log::debug;

use crate::loader::{get_app_info, get_num_app, AppInfo};
use crate::mm::{
    kernel_stack_position, MemorySet, PhysPageNum, VirtAddress, KERNEL_SPACE, TRAP_CONTEXT,
};
use crate::println;
use crate::sbi::shut_down;
use crate::sync::up::UPSafeCell;
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

use super::context::TaskContext;
use super::pid::{KernelStack, PIDHandle};
use super::switch::__switch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    #[allow(unused)]
    UnInit,
    READY,
    RUNNING,
    EXITED(i32),
}

pub struct TaskControlBlock {
    pid: PIDHandle,
    pub status: TaskStatus,
    cx: TaskContext,
    app_info: AppInfo,
    pub inner: Option<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    // the stack field is used and for GC
    #[allow(unused)]
    stack: KernelStack,
    mem_set: MemorySet,
    trap_ctx_ppn: PhysPageNum,
    // base_size to allow brk
    #[allow(unused)]
    base_size: usize,
}

impl TaskControlBlock {
    pub fn new(app: AppInfo) -> Self {
        let (mem_set, usp, entry) = MemorySet::new_app_from_elf(&app.mem);
        let trap_ctx_ppn = mem_set
            .page_table
            .translate(VirtAddress::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let status = TaskStatus::READY;
        let pid = PIDHandle::new();
        let kstack = KernelStack::new(&pid);
        let ksp = kstack.get_top();
        let block_inner = TaskControlBlockInner {
            stack: kstack,
            mem_set,
            trap_ctx_ppn,
            base_size: usp,
        };
        let trap_ctx = block_inner.get_trap_ctx();
        let block = Self {
            pid,
            status,
            app_info: app,
            cx: TaskContext::goto_trap_return(ksp),
            inner: Some(block_inner),
        };
        *trap_ctx = TrapContext::init_new_app(
            usp,
            entry,
            KERNEL_SPACE.exclusive_access().page_table.token(),
            ksp,
            trap_handler as usize,
        );
        block
    }
    pub fn get_pid(&self)->usize{
        self.pid.0
    }
    pub fn get_task_ctx_ptr(&mut self) -> *mut TaskContext {
        &mut self.cx
    }
    pub fn get_trap_ctx(&self) -> Option<&'static mut TrapContext> {
        self.inner.as_ref().map(|b| b.get_trap_ctx())
    }
    pub fn get_mem(&self) -> Option<&MemorySet> {
        self.inner.as_ref().map(|b| &b.mem_set)
    }

    pub fn get_app_info(&self)->&AppInfo{
        &self.app_info
    }
}

fn new_task(app: AppInfo)->UPSafeCell<TaskControlBlock>{
    unsafe{
        UPSafeCell::new(TaskControlBlock::new(app))
    }

}

impl TaskControlBlockInner {
    fn get_trap_ctx(&self) -> &'static mut TrapContext {
        self.trap_ctx_ppn.get_mut()
    }
}

pub struct TaskManager {
    tasks: collections::VecDeque<Arc<UPSafeCell<TaskControlBlock>>>
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> = {
        let num_app = get_num_app();
        let mut tm = TaskManager{tasks: VecDeque::new()};
        for i in 0..num_app {
            tm.add(Arc::new(new_task(get_app_info(i))));
        }
        unsafe {
            UPSafeCell::new(
                tm
            )
        }
    };
}

impl TaskManager {
    pub fn add(&mut self, tcb: Arc<UPSafeCell<TaskControlBlock>>){
        self.tasks.push_back(tcb);
    }
    pub fn fetch(&mut self)->Option<Arc<UPSafeCell<TaskControlBlock>>>{
        self.tasks.pop_front()
    }
}
