use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{println, sync::UCell};

use super::address::{PhysAddress, PhysPageNum};

pub const MEMORY_END: usize = 0x80800000; //8MiB

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn free(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    current: PhysPageNum,
    end: PhysPageNum,
    recycle: Vec<PhysPageNum>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0.into(),
            end: 0.into(),
            recycle: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycle.pop() {
            return Some(ppn);
        }
        if self.current == self.end {
            return None;
        }
        let ppn = self.current;
        self.current.0 += 1;
        Some(ppn)
    }
    fn free(&mut self, ppn: PhysPageNum) {
        if ppn >= self.current || self.recycle.iter().find(|&&v| v == ppn).is_some() {
            panic!("double free frame {:#x}", ppn.0)
        }
        self.recycle.push(ppn);
    }
}

impl StackFrameAllocator {
    fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l;
        self.end = r;
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    static ref FRAME_ALLOCATOR: UCell<FrameAllocatorImpl> =
        unsafe { UCell::new(StackFrameAllocator::new()) };
}

pub fn init() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddress::from(ekernel as usize).ceil(),
        PhysAddress::from(MEMORY_END).floor(),
    );
}

#[derive(Debug)]
pub struct FrameGuard {
    pub ppn: PhysPageNum,
}

impl Drop for FrameGuard {
    fn drop(&mut self) {
        frame_free(self.ppn);
    }
}

impl FrameGuard {
    fn new(ppn: PhysPageNum) -> Self {
        ppn.bytes_mut().fill(0);
        Self { ppn }
    }
}

pub fn frame_new() -> Option<FrameGuard> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(|ppn| FrameGuard::new(ppn))
}

pub fn frame_free(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().free(ppn);
}

#[allow(unused)]
pub fn test_frame_alloc() {
    let mut vs = Vec::new();
    for _ in 0..5 {
        let f = frame_new().unwrap();
        println!("alloc new frame: {:?}", f);
        vs.push(f);
    }
    drop(vs);
    for _ in 0..5 {
        let f = frame_new().unwrap();
        println!("alloc new frame: {:?}", f)
    }
}
