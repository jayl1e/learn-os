use crate::mm::Writer;
use crate::task::{exit_current_task, get_current_app, get_current_pid, get_current_token, suspend_current_task};
use crate::{mm, println, timer};

#[allow(unreachable_code)]
pub fn sys_exit(code: i32) -> ! {
    println!("[kernel] process exit with code: {}", code);
    exit_current_task(code);
    panic!("should not run here")
}

pub fn sys_get_task_info(ptr: *mut u8, len: usize) -> isize {
    let mut buf = mm::UserBufMut::new(get_current_token(), ptr, len);
    let name = get_current_app().name;
    let result = buf.write(name.as_bytes());
    match result {
        Err(e) => {
            println!("write task info error: {}", e.msg);
            return -1;
        }
        Ok(writen) => {
            if writen != name.len() {
                return -2;
            } else {
                return writen as isize;
            }
        }
    }
}

pub fn sys_yield() -> isize {
    suspend_current_task();
    0
}

pub fn sys_get_time() -> isize {
    timer::get_time_ms() as isize
}

pub fn sys_get_pid()->isize{
    get_current_pid() as isize
}