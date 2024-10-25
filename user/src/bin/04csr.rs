#![no_std]
#![no_main]
use riscv::register::sstatus;
use user_lib::println;

#[no_mangle]
fn main() -> i32 {
    println!("set the csr");
    println!("should be killed");
    unsafe {
        sstatus::set_spp(sstatus::SPP::User);
    }
    0
}
