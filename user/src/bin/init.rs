#![no_std]
#![no_main]

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    match fork() {
        0 => {
            exec_shell();
        }
        _ => {
            init_loop();
        }
    }
    0
}

fn exec_shell() {
    let v = exec("user_shell");
    if v != 0 {
        panic!("exec user shell failed")
    }
}

fn init_loop() {
    let mut exit_code = 0;
    loop {
        let pid = wait(&mut exit_code);
        if pid > 0 {
            println!("accept exit code {} from {}", exit_code, pid);
        } else {
            panic!("bad wait pid code");
        }
        yield_();
    }
}
