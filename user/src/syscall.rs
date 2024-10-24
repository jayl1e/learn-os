use core::arch::asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_GET_TASKINFO: usize = 94;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

pub fn sys_write(fd:usize, buf: &[u8])->isize{
    syscall(SYSCALL_WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}

pub fn sys_exit(code:i32)->isize{
    syscall(SYSCALL_EXIT, [code as usize,0,0])
}

pub fn sys_get_task_info(name_buf: &mut [u8])->isize{
    let ptr = name_buf.as_ptr();
    syscall(SYSCALL_GET_TASKINFO, [ptr as usize, name_buf.len() as usize, 0])
}

pub fn sys_yield()->isize{
    syscall(SYSCALL_YIELD, [0;3])
}

pub fn sys_get_time()->isize{
    syscall(SYSCALL_GET_TIME, [0;3])
}

fn syscall(id:usize, args: [usize;3])->isize{
    let mut ret:isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        )
    }
    ret
}

