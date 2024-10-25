#![no_std]
#![no_main]

use core::{arch::asm, ptr};

use user_lib::{self, println};

#[no_mangle]
fn main() -> i32 {
    foo()
}

pub fn foo() -> i32 {
    bar(1)
}

pub fn bar(x: i32) -> i32 {
    println!("going to print stack");
    unsafe {
        print_stack();
    }
    println!("end print stack");
    x + 1
}

pub unsafe fn print_stack() {
    let mut fp: *const usize;
    let mut pc: *const usize;
    asm!("mv {}, fp", out(reg) fp);
    asm!("auipc {}, 0", out(reg) pc);
    while fp != ptr::null() {
        println!("pc: {:p}, fp: {:p}", pc, fp);
        let ra_ptr = fp.sub(1);
        pc = ra_ptr.read_volatile() as *const usize;
        let fp_ptr = ra_ptr.sub(1);
        fp = fp_ptr.read_volatile() as *const usize;
    }
}
