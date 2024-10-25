use buddy_system_allocator::LockedHeap;

const USER_HEAP_SIZE: usize = 0x10000;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

static mut USER_HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[allow(static_mut_refs)]
pub fn init_heap() {
    unsafe {
        let start = USER_HEAP_SPACE.as_ptr() as usize;
        let size = USER_HEAP_SPACE.len();
        HEAP_ALLOCATOR.lock().init(start, size);
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout {:?}", layout)
}
