use core::str;

use crate::{mm::translate_byte_buffer, print, task::get_current_token};

const STDOUT: usize = 1;
pub fn sys_write(fd: usize, address: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => {
            let buffers = translate_byte_buffer(get_current_token(), address, len);
            for buf in buffers{
                unsafe {
                    print!("{}", str::from_utf8_unchecked(buf));
                }
            }
            len as isize
        }
        _ => -1,
    }
}
