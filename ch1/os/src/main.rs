#![no_std]
#![no_main]
use core::arch::global_asm;

mod lang_items;
global_asm!(include_str!("entry.asm"));


fn clear_bss(){
    extern "C"{
        fn sbss();
        fn ebss();
    }
    (sbss as usize .. ebss as usize).for_each(|a|{
        unsafe{
            (a as *mut u8).write_volatile(0)
        }
    });
}

mod sbi;
mod console;

use sbi::shutdown;

#[no_mangle]
pub fn rust_main()->!{
    clear_bss();
    println!("Hello World");
    shutdown();
}