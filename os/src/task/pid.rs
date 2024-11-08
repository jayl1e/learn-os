use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{
    mm::{kernel_stack_position, MapPermission, VirtAddress, KERNEL_SPACE},
    sync::UCell,
};

struct PIDAllocator {
    start: usize,
    recycle: Vec<usize>,
}

impl PIDAllocator {
    fn new() -> Self {
        Self {
            start: 0,
            recycle: Vec::new(),
        }
    }
    fn alloc(&mut self) -> usize {
        match self.recycle.pop() {
            Some(v) => v,
            None => {
                self.start += 1;
                self.start
            }
        }
    }
    fn free(&mut self, pid: usize) {
        self.recycle.push(pid);
    }
}

lazy_static! {
    static ref PID_ALLOCATOR: UCell<PIDAllocator> =
        unsafe { UCell::new(PIDAllocator::new()) };
}

pub struct PIDHandle(pub usize);

impl PIDHandle {
    pub fn new() -> Self {
        Self(PID_ALLOCATOR.exclusive_access().alloc())
    }
}

impl Drop for PIDHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.exclusive_access().free(self.0);
    }
}

pub struct KernelStack {
    bottom: VirtAddress,
    top: VirtAddress,
}

impl KernelStack {
    pub fn new(pid: &PIDHandle) -> Self {
        let (left, right) = kernel_stack_position(pid.0);
        let s = Self {
            bottom: left.into(),
            top: right.into(),
        };
        KERNEL_SPACE.exclusive_access().insert_frame(
            s.bottom,
            s.top,
            MapPermission::R | MapPermission::W,
        );
        s
    }
    pub fn get_top(&self) -> usize {
        self.top.0
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        KERNEL_SPACE
            .exclusive_access()
            .remove_frame(self.bottom)
            .unwrap()
    }
}
