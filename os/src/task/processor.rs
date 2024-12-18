use core::cell::RefMut;
use lazy_static::lazy_static;

use alloc::sync::Arc;
use log::debug;

use crate::{
    loader::AppInfo,
    println,
    sbi::shut_down,
    sync::UCell,
    task::{switch::__switch, task::get_init_proc},
    trap::context::TrapContext,
};

use super::{
    context::TaskContext,
    task::{fork, TaskControlBlock, TaskManager, TaskStatus, TASK_MANAGER},
};

struct Processor {
    pub current: Option<Arc<UCell<TaskControlBlock>>>,
    pub idle_ctx: TaskContext,
    pub tm: &'static UCell<TaskManager>,
}

impl Processor {
    fn new() -> Self {
        Self {
            current: None,
            idle_ctx: TaskContext::zero_init(),
            tm: &TASK_MANAGER,
        }
    }
    fn get_idle_ctx(&mut self) -> *mut TaskContext {
        &mut self.idle_ctx as *mut TaskContext
    }

    fn current_mut(&mut self) -> Option<RefMut<'_, TaskControlBlock>> {
        self.current.as_mut().map(|t| t.exclusive_access())
    }
    fn current(&self) -> Option<RefMut<'_, TaskControlBlock>> {
        self.current.as_ref().map(|t| t.exclusive_access())
    }

    fn mark_current_task_exited(&mut self, code: i32) {
        let mut t = self.current_mut().unwrap();
        println!("[kernel] process {} exit with code: {}", t.get_pid(), code);
        t.status = TaskStatus::EXITED(code);
        t.inner = None;
        if t.parent.is_none() {
            println!("[kernel] init exited")
        } else {
            for kid in &t.children {
                let initproc = get_init_proc();
                kid.exclusive_access()
                    .parent
                    .replace(Arc::downgrade(&initproc));
                initproc.exclusive_access().children.push(kid.clone());
            }
        }
    }
    fn mark_current_task_suspend(&mut self) {
        let mut t = self.current_mut().unwrap();
        t.status = TaskStatus::READY;
    }

    pub fn get_current_token(&self) -> usize {
        self.current()
            .map_or(0, |t| t.get_mem().unwrap().page_table.token())
    }

    fn get_current_trap_cx(&mut self) -> &'static mut TrapContext {
        self.current_mut().unwrap().get_trap_ctx().unwrap()
    }
}

lazy_static! {
    static ref PROCESSOR: UCell<Processor> = unsafe { UCell::new(Processor::new()) };
}

pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        let mut tm = processor.tm.exclusive_access();
        if let Some(old) = processor.current.take() {
            let old_status = old.exclusive_access().status;
            match old_status {
                TaskStatus::READY => {
                    tm.add(old);
                }
                _ => {}
            }
        }
        if let Some(next) = tm.fetch() {
            debug!(
                "[kernel] scheduling pid {}",
                next.exclusive_access().get_pid()
            );
            processor.current = Some(next);
            let mut c = processor.current_mut().unwrap();
            c.status = TaskStatus::RUNNING;
            let nxt = c.get_task_ctx_ptr();
            drop(c);
            let cur = processor.get_idle_ctx();
            drop(tm);
            drop(processor);

            unsafe {
                __switch(cur, nxt);
            }
        } else {
            println!("[kernel] all apps exited, will shutdown");
            shut_down(false)
        }
    }
}

pub fn get_current_app() -> AppInfo {
    PROCESSOR
        .exclusive_access()
        .current()
        .unwrap()
        .get_app_info()
        .clone()
}

pub fn exit_current_task(code: i32) -> ! {
    mark_current_task_exited(code);
    let mut unused = TaskContext::zero_init();
    schedule(&raw mut unused);
    panic!("should not run here")
}

pub fn suspend_current_task() {
    let mut p = PROCESSOR.exclusive_access();
    p.mark_current_task_suspend();
    let cur = p.current().unwrap().get_task_ctx_ptr();
    drop(p);
    schedule(cur);
}

fn mark_current_task_exited(code: i32) {
    PROCESSOR.exclusive_access().mark_current_task_exited(code)
}

fn schedule(old_task_ctx: *mut TaskContext) {
    let mut p = PROCESSOR.exclusive_access();
    let idle_ctx = p.get_idle_ctx();
    drop(p);
    unsafe {
        __switch(old_task_ctx, idle_ctx);
    }
}

pub fn get_current_token() -> usize {
    PROCESSOR.exclusive_access().get_current_token()
}

pub fn get_current_trap_cx() -> &'static mut TrapContext {
    PROCESSOR.exclusive_access().get_current_trap_cx()
}

pub fn fork_current() -> usize {
    let src = PROCESSOR.exclusive_access().current.clone().unwrap();
    let child = fork(src);
    let c = child.exclusive_access();
    c.get_trap_ctx().unwrap().registers[10] = 0; // a0 = 0 for forked child
    let pid = c.get_pid();
    drop(c);
    TASK_MANAGER.exclusive_access().add(child);
    pid
}

pub fn exec_current(app: AppInfo) {
    let mut p = PROCESSOR.exclusive_access();
    let mut t = p.current_mut().unwrap();
    t.exec(app);
}

pub fn get_current_task() -> Option<Arc<UCell<TaskControlBlock>>> {
    PROCESSOR.exclusive_access().current.clone()
}
