#![no_std]
#![no_main]

mod sbi;
mod console;
mod lang_items;
mod logging;
mod sync;
mod batch;
mod syscall;
mod trap;



use log::*;
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.asm"));

#[no_mangle]
#[allow(unreachable_code)]
fn rust_main()->!{
    clear_bss();
    
    logging::init();
    batch::init();
    trap::init();
    println!("[kernel] hello world");
    batch::run_next_app();
    trace!("trace log");
    debug!("debug log");
    info!("info log");
    warn!("warning log");
    error!("error log");
    sbi::shut_down(false);
    panic!("should not run here");
}

fn clear_bss(){
    extern "C"{
        fn sbss();
        fn ebss();
    }
    (sbss as usize .. ebss as usize).for_each(|a|{
        unsafe {
            (a as *mut u8).write_volatile(0);
        }
    });
}

