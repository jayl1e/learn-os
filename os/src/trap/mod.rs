pub mod context;

use crate::{
    println,
    syscall::syscall,
    task::{exit_current_task, suspend_current_task},
    timer,
};
use context::TrapContext;
use core::arch::global_asm;
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

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
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
                    exit_current_task();
                }
            }
        }
        Trap::Exception(_) => {
            println!("[kernel] process exception, killing process");
            exit_current_task();
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
    cx
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
