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
extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod shell;
pub mod racoon;

use pi::gpio::Gpio;
use pi::uart::MiniUart;
use pi::console::{CONSOLE, kprint, kprintln};
use racoon::RACOON_STRING;


#[no_mangle]
pub unsafe extern "C" fn kmain() {
    let mut pin_16 = Gpio::new(16).into_output();
    let mut pin_20 = Gpio::new(20).into_output();
    let mut pin_21 = Gpio::new(21).into_output();

    let uart = MiniUart::new();

    let mut pin_16_on = false;
    let mut pin_20_on = false;
    let mut pin_21_on = false;

    kprintln!("{}", RACOON_STRING);

    loop {
        let mut recv = None;
        kprintln!("<-");

        {
            let mut console = CONSOLE.lock();
            recv = Some(console.read_byte());
        }

        if let Some(byte) = recv {
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
        }
    }
}
