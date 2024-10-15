use core::{slice, str};

use crate::print;

const STDOUT: usize = 1;
pub fn sys_write(fd: usize, address: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => {
            let slice = unsafe { slice::from_raw_parts(address, len) };
            let s = str::from_utf8(slice).unwrap();
            print!("{}", s);
            len as isize
        }
        _ => -1,
    }
}
