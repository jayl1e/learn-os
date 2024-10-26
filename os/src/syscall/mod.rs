const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_GET_TASKINFO: usize = 94;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 201;
const SYSCALL_FORK: usize = 220;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_GETPID:usize = 172;


mod fs;
mod process;

pub fn syscall(syscall_id: usize, a1: usize, a2: usize, a3: usize) -> Option<isize> {
    match syscall_id {
        SYSCALL_WRITE => Some(fs::sys_write(a1, a2 as *const u8, a3)),
        SYSCALL_EXIT => process::sys_exit(a1 as i32),
        SYSCALL_GET_TASKINFO => Some(process::sys_get_task_info(a1 as *mut u8, a2)),
        SYSCALL_YIELD => Some(process::sys_yield()),
        SYSCALL_GET_TIME => Some(process::sys_get_time()),
        SYSCALL_GETPID => Some(process::sys_get_pid()),
        _ => None,
    }
}
