#![no_std]
#![no_main]
use core::arch::asm;

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    println!("exec privileged instruction");
    println!("should be killed");
    unsafe {
        asm!("sret");
    };
    0
}
