use crate::{batch::{get_current_app, run_next_app}, println};

pub fn sys_exit(code:i32)->!{
    println!("[kernel] process exit with code: {}", code);
    run_next_app()
}

pub fn sys_get_task_info(name_buf: &mut [u8])->isize{
    println!("[kernel] calling get task info");
    let name = get_current_app().name;
    if name_buf.len()<name.len(){
        return -1;
    }
    name_buf[..name.len()].copy_from_slice(name.as_bytes());
    name.len() as isize
}