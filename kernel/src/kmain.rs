#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(never_type)]
#![feature(ptr_internals)]
#![feature(panic_implementation)]
#![feature(nll)]

#[macro_use]
extern crate core;
#[macro_use]
extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;

use std::fmt::Write;
use pi::timer::spin_sleep_ms;
use pi::gpio::Gpio;
use pi::uart::MiniUart;
use console::{kprint, kprintln};


#[no_mangle]
pub unsafe extern "C" fn kmain() {
    let mut pin_16 = Gpio::new(16).into_output();
    let mut pin_20 = Gpio::new(20).into_output();
    let mut pin_21 = Gpio::new(21).into_output();

    let mut uart = MiniUart::new();

    let mut pin_16_on = false;
    let mut pin_20_on = false;
    let mut pin_21_on = false;
    loop {
        let byte = uart.read_byte();
        // uart.write_byte(byte);
        kprint!("{}", byte);
        if pin_16_on {
            pin_16.clear();
            pin_16_on = false;
        } else {
            pin_16.set();
            pin_16_on = true
        }
        if byte == 0x41 {
            if pin_20_on {
                pin_20.clear();
                pin_20_on = false;
            } else {
                pin_20.set();
                pin_20_on = true
            }
        }
        if byte == 0x42 {
            if pin_21_on {
                pin_21.clear();
                pin_21_on = false;
            } else {
                pin_21.set();
                pin_21_on = true
            }
        }
        kprintln!("<-");
        // uart.write_str("<-");
    }
}
