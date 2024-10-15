#![no_std]
#![no_main]

mod console;
mod lang_items;
mod loader;
mod logging;
mod sbi;
mod sync;
mod syscall;
mod task;
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
    loader::init();
    trap::init();
    println!("[kernel] hello");
    trace!("start loading");
    unsafe {
        loader::load_all_apps();
    }
    trace!("start running");
    task::run_first_task();
    sbi::shut_down(false);
    panic!("should not run here");
}

fn clear_bss() {
    extern "C" {
        static sbss: usize;
        static ebss: usize;
    }
    unsafe {
        slice::from_raw_parts_mut(sbss as *mut u8, ebss - sbss).fill(0);
    }
}
