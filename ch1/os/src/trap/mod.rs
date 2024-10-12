pub mod context;

use core::arch::global_asm;
use riscv::register::{scause::{self, Exception, Trap}, stval, stvec};
use context::TrapContext;
use crate::{println,syscall::syscall};
use crate::batch::run_next_app;

global_asm!(include_str!("trap.asm"));

pub fn init(){
    extern "C" {fn __alltraps();}
    unsafe {
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext)->&mut TrapContext{
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall)=>{
            cx.sepc += 4;
            let rt = syscall(cx.registers[17],cx.registers[10],cx.registers[11], cx.registers[12]);
            match rt {
                Some(rt)=>{
                    cx.registers[10] = rt as usize;
                }
                None=>{
                    println!("[kernel] bad syscall, killing process");
                    run_next_app()
                }
            }
        },
        Trap::Exception(_)=>{
            println!("[kernel] process exception, killing process");
            run_next_app()
        },
        Trap::Interrupt(_)=>{
            panic!("unsupported interrupt: scause {:?}, stval {}", scause.cause(), stval)
        }
    }
    cx
}