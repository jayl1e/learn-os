use log::debug;
mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;
mod memory_set;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init();
    debug!("activating paging");
    memory_set::KERNEL_SPACE.exclusive_access().activate();
}

pub use address::{PhysPageNum, VirtAddress};
pub use memory_set::{MemorySet, TRAMPOLINE, TRAP_CONTEXT, kernel_stack_position, KERNEL_SPACE, MapPermission};
pub use page_table::translate_byte_buffer;

#[allow(unused_imports)]
pub use frame_allocator::test_frame_alloc;
#[allow(unused_imports)]
pub use heap_allocator::test_heap;
