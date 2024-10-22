use crate::trap::trap_return;

const KEEP_REGISTER: usize = 12;
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize, //kernel ra
    pub sp: usize, //kernel sp
    s: [usize; KEEP_REGISTER],
}

impl TaskContext {
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; KEEP_REGISTER],
        }
    }

    pub fn goto_trap_return(sp: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp,
            s: [0; KEEP_REGISTER],
        }
    }
}
