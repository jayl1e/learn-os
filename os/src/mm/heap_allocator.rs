use buddy_system_allocator::LockedHeap;

use crate::println;

const KERNEL_HEAP_SIZE: usize = 0x100000;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

static mut KERNEL_HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[allow(static_mut_refs)]
pub fn init_heap() {
    unsafe {
        let start = KERNEL_HEAP_SPACE.as_ptr() as usize;
        let size = KERNEL_HEAP_SPACE.len();
        HEAP_ALLOCATOR.lock().init(start, size);
    }
}

#[alloc_error_handler]
fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout {:?}", layout)
}

#[allow(unused, static_mut_refs)]
pub fn test_heap() {
    use alloc::boxed::Box;
    use alloc::vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    println!(
        "bss is in [{:p}, {:p}]",
        sbss as *const u8, ebss as *const u8
    );
    unsafe {
        println!("heap space is {:?}", KERNEL_HEAP_SPACE.as_ptr_range());
    }
    let v = vec![1, 2, 3];
    println!("v[1] is {} at {:p}", v[1], &v[1]);
    let v = Box::new(3);
    println!("box is at {:p}", &*v);
}
