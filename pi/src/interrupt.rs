use common::IO_BASE;
use console::kprintln;
use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile};

const INT_BASE: usize = IO_BASE + 0xB000 + 0x200;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interrupt {
    Timer1 = 1,
    Timer3 = 3,
    Usb = 9,
    Gpio0 = 49,
    Gpio1 = 50,
    Gpio2 = 51,
    Gpio3 = 52,
    Uart = 57,
}

static INTERRUPTS: [Interrupt;  8] = [
    Interrupt::Timer1,
    Interrupt::Timer3,
    Interrupt::Usb,
    Interrupt::Gpio0,
    Interrupt::Gpio1,
    Interrupt::Gpio2,
    Interrupt::Gpio3,
    Interrupt::Uart
];

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IRQ_BASIC_PENDING: ReadVolatile<u32>,
    IRQ_PENDING_1: ReadVolatile<u32>,
    IRQ_PENDING_2: ReadVolatile<u32>,
    FIQ_CONTROL: Volatile<u32>,
    ENABLE_IRQS_1: Volatile<u32>,
    ENABLE_IRQS_2: Volatile<u32>,
    ENABLE_BASIC_IRQS: Volatile<u32>,
    DISABLE_IRQS_1: Volatile<u32>,
    DISABLE_IRQS_2: Volatile<u32>,
    DISABLE_BASIC_IRQS: Volatile<u32>,
}

/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct Controller {
    registers: &'static mut Registers
}

impl Controller {
    /// Returns a new handle to the interrupt controller.
    pub fn new() -> Controller {
        Controller {
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    /// Enables the interrupt `int`.
    pub fn enable(&mut self, int: Interrupt) {
        if (int as u32) < 32 {
            kprintln!("enabling int {} as u32", int as u32);
            kprintln!("enabling int 0b{:b}", 0x01 << int as u32);
            self.registers.ENABLE_IRQS_1.or_mask(0x01 << int as u32);
        } else {
            self.registers.ENABLE_IRQS_2.or_mask(0x01 << (32 - int as u32 ));
        }
    }

    /// Disables the interrupt `int`.
    pub fn disable(&mut self, int: Interrupt) {
        if (int as u32) < 32 {
            self.registers.DISABLE_IRQS_1.or_mask(0x01 << int as u32);
        } else {
            self.registers.DISABLE_IRQS_2.or_mask(0x01 << (32 - int as u32));
        }
    }

    /// Returns `true` if `int` is pending. Otherwise, returns `false`.
    pub fn is_pending(&self, int: Interrupt) -> bool {
        if (int as u32) < 32 {
            self.registers.IRQ_PENDING_1.has_mask(0x01 << int as u32)
        } else {
            self.registers.IRQ_PENDING_1.has_mask(0x01 << (32 - int as u32))
        }
    }

    pub fn first_pending(&self) -> Option<Interrupt> {
        for i in INTERRUPTS.into_iter() {
            if self.is_pending(*i) {
                return Some(*i);
            }
        }
        None
    }
}
