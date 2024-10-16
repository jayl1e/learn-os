use crate::{println, timer};
use crate::task::{exit_current_task, get_current_app, suspend_current_task};

#[allow(unreachable_code)]
pub fn sys_exit(code: i32) -> ! {
    println!("[kernel] process exit with code: {}", code);
    exit_current_task();
    panic!("should not run here")
}

pub fn sys_get_task_info(name_buf: &mut [u8]) -> isize {
    let name = get_current_app().name;
    if name_buf.len() < name.len() {
        return -1;
    }
    name_buf[..name.len()].copy_from_slice(name.as_bytes());
    name.len() as isize
}

pub fn sys_yield() -> isize {
    suspend_current_task();
    0
}

pub fn sys_get_time()->isize{
    timer::get_time_us() as isize
}