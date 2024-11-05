#![no_std]
#![no_main]

use user_lib::*;

#[no_mangle]
fn main() -> i32 {
    println!("init process start");
    exec_hello();
    exec_shell();
    //init_loop();
    println!("init exit");
    0
}

fn exec_shell() {
    if fork()==0{
        let v = exec("user_shell");
        if v != 0 {
            panic!("exec user shell failed")
        }
    }
}

fn exec_hello(){
    if fork()==0{
        let v = exec("00hello");
        if v != 0 {
            panic!("exec user shell failed")
        }
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
