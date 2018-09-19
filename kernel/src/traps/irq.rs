use pi::interrupt::Interrupt;
use pi::console::kprintln;
use pi::timer;
use process::TICK;

use traps::TrapFrame;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    match interrupt {
        Interrupt::Timer1 => {
            kprintln!("TICK");
            timer::tick_in(TICK);
        },
        Interrupt::Timer3 => {
            kprintln!("IRQ from Timer3 unhandled!");
        }
        _ => {
            kprintln!("IRQ from non-timer");
        }
    }
}
