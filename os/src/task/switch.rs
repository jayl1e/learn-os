use core::arch::global_asm;

use super::context::TaskContext;

global_asm!(include_str!("switch.asm"));

extern "C" {
    pub fn __switch(cur: *mut TaskContext, nxt: *const TaskContext);
}
