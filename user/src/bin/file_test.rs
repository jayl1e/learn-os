#![no_std]
#![no_main]

use user_lib::{close, open, println, read, write, OpenFlags};

#[no_mangle]
fn main() -> i32 {
    let filename = "sample.txt";
    let content = "hello str";
    let fd = open(filename, OpenFlags::WRONLY | OpenFlags::CREATE);
    if fd < 0 {
        println!("can not open file to write");
        return 1;
    }
    let fd = fd as usize;
    let written = write(fd, content.as_bytes());
    if written != content.len() as isize {
        println!("can not write to file");
        return 1;
    }
    close(fd);
    let fd = open(filename, OpenFlags::RDONLY);
    if fd < 0 {
        println!("can not open file to read");
        return 1;
    }
    let fd = fd as usize;
    let mut buf = [0; 128];
    let readed = read(fd, &mut buf) as usize;
    if readed != content.len() {
        println!("bad read");
        return 1;
    }
    if !content.as_bytes().eq(&buf[..readed]) {
        println!("readed content not consist");
        return 1;
    }
    0
}
