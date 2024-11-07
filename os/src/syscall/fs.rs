use core::str;

use crate::{
    mm::{translate_ptr_mut, Reader, UserBuf},
    print, println,
    sbi::console_get_char,
    task::get_current_token,
};

use super::{EAGAIN, EBADARG};

const STDIN: usize = 0;
const STDOUT: usize = 1;
const BUFFER_SIZE: usize = 2048;
pub fn sys_write(fd: usize, address: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => {
            let mut buffers = UserBuf::new(get_current_token(), address, len);
            let mut buf = [0; BUFFER_SIZE];
            let mut written = 0;
            loop {
                match buffers.read(&mut buf) {
                    Err(err) => {
                        println!("read from user failed: {}", err.msg);
                        return -1;
                    }
                    Ok(readed) => {
                        let valid_buf = &buf[..readed];
                        unsafe {
                            print!("{}", str::from_utf8_unchecked(valid_buf));
                        }
                        written += readed;
                        if readed < buf.len() {
                            break;
                        }
                    }
                }
            }
            written as isize
        }
        _ => -1,
    }
}
pub fn sys_read(fd: usize, address: *mut u8, len: usize) -> isize {
    match fd {
        STDIN => {
            let c = console_get_char();
            if c == 0 {
                EAGAIN
            } else {
                if len < 1 {
                    return EBADARG;
                }
                match translate_ptr_mut(address, get_current_token()) {
                    Some(ptr) => {
                        *ptr = c as u8;
                        1
                    }
                    None => -1,
                }
            }
        }
        _ => -1,
    }
}
