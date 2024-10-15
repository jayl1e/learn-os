const KEEP_REGISTER: usize = 12;
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
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

    pub fn goto_restore(sp: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: sp,
            s: [0; KEEP_REGISTER],
        }
    }
}
