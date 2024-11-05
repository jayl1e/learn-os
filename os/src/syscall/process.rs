use core::str;

use crate::loader::get_app_info_by_name;
use crate::mm::{Reader, Writer};
use crate::task::{
    exec_current, exit_current_task, fork_current, get_current_app, get_current_pid, get_current_token, suspend_current_task
};
use crate::{mm, println, timer};

#[allow(unreachable_code)]
pub fn sys_exit(code: i32) -> ! {
    exit_current_task(code);
    panic!("should not run here")
}

pub fn sys_get_task_info(ptr: *mut u8, len: usize) -> isize {
    let mut buf = mm::UserBufMut::new(get_current_token(), ptr, len);
    let name = get_current_app().name;
    let result = buf.write(name.as_bytes());
    match result {
        Err(e) => {
            println!("[kernel] write task info error: {}", e.msg);
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

pub fn sys_get_pid() -> isize {
    get_current_pid() as isize
}

pub fn sys_fork()->isize{
    fork_current() as isize
}

const PATH_LENGTH_LIMIT:usize=128;

pub fn sys_exec(ptr: *mut u8)->isize{
    let mut path_buf = [0u8; PATH_LENGTH_LIMIT];
    let mut wptr = 0;
    for c in mm::iter_from_user_ptr(ptr, get_current_token()){
        if c!=0 && wptr<PATH_LENGTH_LIMIT{
            path_buf[wptr]=c;
            wptr+=1;
        }else{
            break;
        }
    }
    if wptr >= PATH_LENGTH_LIMIT{
        return -2;
    }

    let name = match str::from_utf8(&path_buf[..wptr]){
        Ok(s)=>s,
        Err(e)=>{
            println!("[kernel] read path while exec error: bad utf8 at {}", e.valid_up_to());
            return -2
        }
    };

    let app = get_app_info_by_name(name);
    match app{
        Some(app)=>{
            exec_current(app);
            0
        },
        None=>-1
    }
}
