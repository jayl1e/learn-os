use core::str;

use crate::{
    mm::{Reader, UserBuf},
    print, println,
    task::get_current_token,
};

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
