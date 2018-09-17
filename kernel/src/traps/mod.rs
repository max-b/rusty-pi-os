mod irq;
mod trap_frame;
mod syndrome;
mod syscall;

use pi::interrupt::{Controller, Interrupt};

use shell::shell;

pub use self::trap_frame::TrapFrame;

use aarch64;
use pi::console::kprintln;
use self::syndrome::Syndrome;
use self::irq::handle_irq;
use self::syscall::handle_syscall;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    kprintln!("info: {:#?}", info);
    kprintln!("esr: {:#x?}", esr);
    kprintln!("tf: {:#?}", tf);

    let syndrome = Syndrome::from(esr);

    kprintln!("syndrome = {:#x?}", syndrome);
    if let Syndrome::Brk(break_num) = syndrome {
        kprintln!("entering break num: {:x}", break_num);
        shell("$!> ");
        shell(&format!("{} $!> ", break_num));
    } else if let Syndrome::Breakpoint = syndrome {
        kprintln!("entering breakpoint");
        shell("$!> ");
    }

    loop {
        aarch64::nop();
    }
}
