mod context;

pub use context::TrapContext;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

use crate::{
    syscall::syscall,
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::set_next_trigger,
};

core::arch::global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(
            __alltraps as usize,
            riscv::register::utvec::TrapMode::Direct,
        );
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[12], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            error!(
                "[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                 stval, cx.sepc);
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] IllegalInstruction in application, core dumped.");
            exit_current_and_run_next();
        }
        scause::Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause.cause(),
                stval,
            )
        }
    }

    todo!()
}

pub(crate) fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
