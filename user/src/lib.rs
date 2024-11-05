#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]

extern crate alloc;
use syscall::{sys_get_time, sys_yield};

pub mod console;
mod heap;
mod lang_items;
mod syscall;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    heap::init_heap();
    exit(main());
    unreachable!("should exit after main")
}

fn clear_bss() {
    extern "C" {
        // use fn because we want to access there as pointer
        // simple usize will read data there
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as *mut u8, ebss as usize - sbss as usize).fill(0);
    }
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("can not find main")
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    syscall::sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    syscall::sys_exit(exit_code)
}
pub fn get_task_info(name_buf: &mut [u8]) -> Option<&str> {
    let l = syscall::sys_get_task_info(name_buf);
    if l < 0 {
        return None;
    }
    let name = core::str::from_utf8(&name_buf[..(l as usize)]).unwrap();
    Some(name)
}

pub fn yield_() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}

const EAGAIN: isize = -2;
pub fn wait(code: &mut i32) -> isize {
    loop {
        match syscall::sys_waitpid(-1, code as *mut i32) {
            EAGAIN => {
                yield_();
            }
            pid => return pid,
        }
    }
}
pub fn wait4(pid: usize, code: &mut i32) -> isize {
    if pid > isize::MAX as usize {
        return -1;
    }
    loop {
        match syscall::sys_waitpid(pid as isize, code as *mut i32) {
            EAGAIN => {
                yield_();
            }
            other => return other,
        }
    }
}

pub fn fork() -> isize {
    syscall::sys_fork()
}

pub fn exec(path: &str) -> isize {
    let mut buf: [u8; 128] = [0; 128];
    let len = path.len();
    if len >= buf.len() {
        println!("path is too long");
        return -1;
    }
    buf[..len].copy_from_slice(path.as_bytes());
    buf[len] = 0;
    syscall::sys_exec(buf.as_ptr())
}

const FD_STDIN: usize = 0;

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    loop {
        match syscall::sys_read(fd, buf) {
            EAGAIN => {
                yield_();
                continue;
            }
            other => {
                return other;
            }
        }
    }
}

pub fn get_char() -> Option<u8> {
    let mut buf = [0u8; 1];
    match read(FD_STDIN, &mut buf) {
        0=>None,
        1=>Some(buf[0]),
        _other =>{
            panic!("read stdin failed")
        }
    }
}

pub fn get_pid()->isize{
    syscall::sys_get_pid()
}
