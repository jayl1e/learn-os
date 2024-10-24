#![no_std]
#![no_main]

use user_lib::{self, println, yield_};

#[no_mangle]
fn main()->i32{
    println!("hello app0");
    yield_();
    println!("hello app0 again");
    let mut namebuf = [0u8;128];
    let name = user_lib::get_task_info(&mut namebuf[..]);
    match name{
        Some(name)=>{
            println!("my app name is: {}, I'll run some time", name);
            spend_some_time();
            println!("00hello is going to exit")
        }
        None=>{
            println!("get name failed");
        }
    }
    0
}

fn spend_some_time(){
    let mut v=0;
    let ptr = &raw mut v;
    for i in 0..100000000{
        unsafe {
            ptr.write_volatile(i);
        }
    }
}