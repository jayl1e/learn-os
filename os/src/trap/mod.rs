pub mod context;

use crate::{
    mm::{TRAMPOLINE, TRAP_CONTEXT},
    println,
    syscall::syscall,
    task::{exit_current_task, get_current_token, get_current_trap_cx, suspend_current_task},
    timer,
};
use context::TrapContext;
use core::arch::{asm, global_asm};
use log::debug;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.asm"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
}

fn set_trap_from_kernel() {
    unsafe {
        stvec::write(trap_from_kernel as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
fn trap_from_kernel() -> ! {
    panic!("no trap from kernel")
}
const ECODE_BAD_PROCESS_HEHAVIOR:i32 = 137;

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> ! {
    set_trap_from_kernel();
    let cx = get_current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            let rt = syscall(
                cx.registers[17],
                cx.registers[10],
                cx.registers[11],
                cx.registers[12],
            );
            match rt {
                Some(rt) => {
                    cx.registers[10] = rt as usize;
                }
                None => {
                    println!("[kernel] bad syscall, killing process");
                    exit_current_task(ECODE_BAD_PROCESS_HEHAVIOR);
                }
            }
        }
        Trap::Exception(_) => {
            println!("[kernel] process exception, killing process");
            exit_current_task(ECODE_BAD_PROCESS_HEHAVIOR);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            timer::set_next_trigger();
            debug!("[kernel] clock interrupted");
            suspend_current_task();
        }
        Trap::Interrupt(_) => {
            panic!(
                "unsupported interrupt: scause {:?}, stval {}",
                scause.cause(),
                stval
            )
        }
    }
    trap_return()
}

fn set_trap_from_user() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, stvec::TrapMode::Direct);
    }
}

#[allow(unreachable_code)]
#[no_mangle]
pub fn trap_return() -> ! {
    set_trap_from_user();
    let trap_ctx_ptr = TRAP_CONTEXT;
    let user_satp = get_current_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_ctx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
    panic!("unreachable after back to user")
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
