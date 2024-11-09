#![no_std]
#![no_main]

use user_lib::{self, get_pid, get_time, println, yield_};

#[no_mangle]
fn main() -> i32 {
    println!("I am going to sleep 100ms");
    let start = get_time();
    let end = start + 100;
    let mut now = get_time();
    while now < end {
        println!("now is {}", now);
        yield_();
        now = get_time();
        spend_some_time();
    }
    let pid = get_pid();
    println!("I am pid: {}", pid);
    println!("sleep enough, now is {}", now);
    0
}

fn spend_some_time() {
    let mut v = 0;
    let ptr = &raw mut v;
    for i in 0..30000000 {
        unsafe {
            ptr.write_volatile(i);
        }
    }
}
