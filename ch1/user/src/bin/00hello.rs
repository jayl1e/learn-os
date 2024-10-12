#![no_std]
#![no_main]

use user_lib::{self, println};

#[no_mangle]
fn main()->i32{
    println!("hello world");
    0
}