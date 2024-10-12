use crate::{batch::run_next_app, println};

pub fn sys_exit(code:i32)->!{
    println!("[kernel] process exit with code: {}", code);
    run_next_app()
}