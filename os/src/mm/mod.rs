use log::debug;
mod address;
mod frame_allocator;
mod heap_allocator;
mod io;
mod memory_set;
mod page_table;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init();
    debug!("activating paging");
    memory_set::KERNEL_SPACE.exclusive_access().activate();
}

pub use address::{PhysPageNum, VirtAddress};
pub use io::{iter_from_user_ptr, translate_ptr_mut, Reader, UserBuf, UserBufMut, Writer};
pub use memory_set::{
    kernel_stack_position, MapPermission, MemorySet, KERNEL_SPACE, TRAMPOLINE, TRAP_CONTEXT,
};

#[allow(unused_imports)]
pub use frame_allocator::test_frame_alloc;
#[allow(unused_imports)]
pub use heap_allocator::test_heap;
