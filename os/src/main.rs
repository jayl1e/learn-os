#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod config;
mod console;
mod lang_items;
mod logging;
mod mm;
mod sbi;
mod sync;

mod loader;
mod syscall;
mod task;
mod timer;
mod trap;

use core::{arch::global_asm, slice};
use log::*;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.asm"));

#[no_mangle]
#[allow(unreachable_code)]
fn rust_main() -> ! {
    clear_bss();
    logging::init();
    debug!("[kernel] init mm");
    mm::init();
    debug!("[kernel] init loader");
    loader::init();
    debug!("[kernel] init trap");
    trap::init();
    // for test
    // mm::heap_allocator::test_heap();
    //mm::test_frame_alloc();
    //sbi::shut_down(false);

    println!("[kernel] hello going to run apps");
    trace!("start loading");
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    trace!("start running");
    task::run_tasks();
    sbi::shut_down(false);
    panic!("should not run here");
}

fn clear_bss() {
    extern "C" {
        // use fn because we want to access there as pointer
        // simple usize will read data there
        fn sbss();
        fn ebss();
    }
    unsafe {
        slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize).fill(0);
    }
}
