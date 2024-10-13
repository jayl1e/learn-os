#![no_std]
#![no_main]

use user_lib::{self, println};

#[no_mangle]
fn main()->i32{
    println!("hello");
    println!("world");
    let mut namebuf = [0u8;128];
    let name = user_lib::get_task_info(&mut namebuf[..]);
    match name{
        Some(name)=>{
            println!("my app name is: {}", name);
        }
        None=>{
            println!("get name failed");
        }
    }
    0
}