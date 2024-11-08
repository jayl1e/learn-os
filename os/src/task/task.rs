use alloc::collections::vec_deque::VecDeque;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::loader::{get_app_info_by_name, AppInfo};
use crate::mm::{MemorySet, PhysPageNum, VirtAddress, KERNEL_SPACE, TRAP_CONTEXT};
use crate::sync::UCell;
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

use super::context::TaskContext;
use super::pid::{KernelStack, PIDHandle};

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
    pub parent: Option<Weak<UCell<TaskControlBlock>>>,
    pub children: Vec<Arc<UCell<TaskControlBlock>>>,
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
    pub fn exec(&mut self, app: AppInfo) {
        let (mem_set, usp, entry) = MemorySet::new_app_from_elf(&app.mem);
        self.app_info = app;
        let trap_ctx_ppn = mem_set
            .page_table
            .translate(VirtAddress::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let inner = self.inner.as_mut().unwrap();
        inner.mem_set = mem_set;
        inner.trap_ctx_ppn = trap_ctx_ppn;
        inner.base_size = usp;
        let ksp = inner.stack.get_top();
        let trap_ctx = inner.get_trap_ctx();
        *trap_ctx = TrapContext::init_new_app(
            usp,
            entry,
            KERNEL_SPACE.exclusive_access().page_table.token(),
            ksp,
            trap_handler as usize,
        );
        self.cx = TaskContext::goto_trap_return(ksp)
    }
    pub fn get_pid(&self) -> usize {
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

    pub fn exit_code(&self) -> Option<i32> {
        match self.status {
            TaskStatus::EXITED(i) => Some(i),
            _ => None,
        }
    }

    pub fn get_app_info(&self) -> &AppInfo {
        &self.app_info
    }
}

fn new_task(app: AppInfo) -> Arc<UCell<TaskControlBlock>> {
    let status = TaskStatus::READY;
    let pid = PIDHandle::new();
    let kstack = KernelStack::new(&pid);
    let block_inner = TaskControlBlockInner {
        stack: kstack,
        mem_set: MemorySet::bare_new(),
        trap_ctx_ppn: PhysPageNum(0),
        base_size: 0,
    };
    let mut block = TaskControlBlock {
        pid,
        status,
        app_info: app.clone(),
        cx: TaskContext::zero_init(),
        children: Vec::new(),
        parent: None,
        inner: Some(block_inner),
    };
    block.exec(app);
    Arc::new(unsafe { UCell::new(block) })
}

pub fn fork(parent: Arc<UCell<TaskControlBlock>>) -> Arc<UCell<TaskControlBlock>> {
    let mut src = parent.exclusive_access();
    let mem_set = src.inner.as_ref().unwrap().mem_set.fork();
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
        base_size: src.inner.as_ref().unwrap().base_size,
    };
    let block = TaskControlBlock {
        pid,
        status,
        app_info: src.app_info.clone(),
        cx: TaskContext::goto_trap_return(ksp),
        children: Vec::new(),
        parent: Some(Arc::downgrade(&parent)),
        inner: Some(block_inner),
    };
    //every the same except kernel sp
    block.get_trap_ctx().unwrap().kernel_sp = ksp;
    let child = Arc::new(unsafe { UCell::new(block) });
    src.children.push(child.clone());
    child
}

impl TaskControlBlockInner {
    fn get_trap_ctx(&self) -> &'static mut TrapContext {
        self.trap_ctx_ppn.get_mut()
    }
}

pub struct TaskManager {
    tasks: VecDeque<Arc<UCell<TaskControlBlock>>>,
    init_proc: Option<Arc<UCell<TaskControlBlock>>>,
}

lazy_static! {
    pub static ref TASK_MANAGER: UCell<TaskManager> = {
        let tm = TaskManager {
            tasks: VecDeque::new(),
            init_proc: None,
        };
        unsafe { UCell::new(tm) }
    };
}

pub fn add_init_proc() {
    let mut m = TASK_MANAGER.exclusive_access();
    let init_proc = get_app_info_by_name("init");
    match init_proc {
        None => {
            panic!("no init process")
        }
        Some(app) => {
            let init = new_task(app.clone());
            m.add(init.clone());
            m.init_proc.replace(init);
        }
    }
}

pub fn get_init_proc() -> Arc<UCell<TaskControlBlock>> {
    TASK_MANAGER
        .exclusive_access()
        .init_proc
        .as_ref()
        .unwrap()
        .clone()
}

impl TaskManager {
    pub fn add(&mut self, tcb: Arc<UCell<TaskControlBlock>>) {
        self.tasks.push_back(tcb);
    }
    pub fn fetch(&mut self) -> Option<Arc<UCell<TaskControlBlock>>> {
        self.tasks.pop_front()
    }
}
