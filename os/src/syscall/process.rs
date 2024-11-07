use core::str;

use alloc::sync::Arc;

use crate::loader::get_app_info_by_name;
use crate::mm::{translate_ptr_mut, Writer};
use crate::task::{
    exec_current, exit_current_task, fork_current, get_current_app, get_current_task,
    get_current_token, suspend_current_task,
};
use crate::{mm, println, timer};

use super::{EAGAIN, EBADARG, ENOCHILDREN};

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
    let current = get_current_task().unwrap();
    let cur = current.exclusive_access();
    let pid = cur.get_pid() as isize;
    pid
}

pub fn sys_fork() -> isize {
    fork_current() as isize
}

const PATH_LENGTH_LIMIT: usize = 128;

pub fn sys_exec(ptr: *mut u8) -> isize {
    let mut path_buf = [0u8; PATH_LENGTH_LIMIT];
    let mut wptr = 0;
    for c in mm::iter_from_user_ptr(ptr, get_current_token()) {
        if c != 0 && wptr < PATH_LENGTH_LIMIT {
            path_buf[wptr] = c;
            wptr += 1;
        } else {
            break;
        }
    }
    if wptr >= PATH_LENGTH_LIMIT {
        return -2;
    }

    let name = match str::from_utf8(&path_buf[..wptr]) {
        Ok(s) => s,
        Err(e) => {
            println!(
                "[kernel] read path while exec error: bad utf8 at {}",
                e.valid_up_to()
            );
            return -2;
        }
    };

    let app = get_app_info_by_name(name);
    match app {
        Some(app) => {
            exec_current(app);
            0
        }
        None => EBADARG,
    }
}

pub fn sys_waitpid(pid: isize, code_ptr: *mut i32) -> isize {
    let current = get_current_task().unwrap();
    let mut cur = current.exclusive_access();
    if pid != -1
        && cur
            .children
            .iter()
            .all(|k| k.exclusive_access().get_pid() as isize != pid)
    {
        return EBADARG;
    }
    let found = cur.children.iter().enumerate().find(|(_idx, kid)| {
        let k = kid.exclusive_access();
        (pid == -1 || pid == k.get_pid() as isize) && k.exit_code().is_some()
    });
    match found {
        None => {
            if cur.children.len() == 0 {
                ENOCHILDREN
            } else {
                EAGAIN
            }
        }
        Some((idx, _)) => {
            let k = cur.children.swap_remove(idx);
            assert_eq!(1, Arc::strong_count(&k), "exited task should only ref by 1");
            let code = k.exclusive_access().exit_code().unwrap();
            let found = k.exclusive_access().get_pid();
            // get current toke will lock current process, so drop cur
            drop(cur);
            match translate_ptr_mut(code_ptr, get_current_token()) {
                None => {
                    return -1;
                }
                Some(code_ref) => {
                    *code_ref = code;
                }
            }
            found as isize
        }
    }
}
