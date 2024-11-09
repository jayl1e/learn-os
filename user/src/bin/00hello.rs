#![no_std]
#![no_main]

use user_lib::{self, println, yield_};

#[no_mangle]
fn main() -> i32 {
    println!("hello app0");
    yield_();
    println!("hello app0 again");
    let mut namebuf = [0u8; 128];
    let name = user_lib::get_task_info(&mut namebuf[..]);
    match name {
        Some(name) => {
            println!("my app name is: {}, going to exit", name);
        }
        None => {
            println!("get name failed");
        }
    }
    0
}
