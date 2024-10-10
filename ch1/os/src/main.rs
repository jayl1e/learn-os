#![no_std]
#![no_main]

mod sbi;
mod console;
mod lang_items;
mod logging;

use log::*;
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
#[allow(unreachable_code)]
fn rust_main()->!{
    clear_bss();
    
    logging::init();
    println!("hello world");
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

