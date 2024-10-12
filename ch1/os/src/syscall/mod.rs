
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

mod fs;
mod process;

pub fn syscall(syscall_id:usize, a1:usize, a2:usize, a3:usize)->Option<isize>{
    match syscall_id {
        SYSCALL_WRITE =>{Some(fs::sys_write(a1,a2 as *const u8,a3))},
        SYSCALL_EXIT => {process::sys_exit(a1 as i32)}
        _=>{None}
    }
}