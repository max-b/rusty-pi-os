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
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api, global_allocator)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
#[macro_use]
extern crate core;
extern crate pi;
extern crate stack_vec;
extern crate fat32;
extern crate volatile;

pub mod allocator;
pub mod lang_items;
pub mod shell;
pub mod racoon;
pub mod fs;

use volatile::Writeable;
use pi::gpio::Gpio;
use pi::console::{CONSOLE, kprintln};
use pi::framebuffer::Framebuffer;
use racoon::RACOON_STRING;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

#[no_mangle]
#[cfg(not(test))]
pub unsafe extern "C" fn kmain() {
    let mut pin_16 = Gpio::new(16).into_output();
    let mut pin_20 = Gpio::new(20).into_output();
    let mut pin_21 = Gpio::new(21).into_output();

    let mut pin_16_on = false;
    let mut pin_20_on = false;
    let mut pin_21_on = false;

    kprintln!("{}", RACOON_STRING);

    let framebuffer = Framebuffer::new().expect("Error creating new framebuffer");

    let mut channel_counter = 0;
    loop {

        kprintln!("<-");

        let byte = {
            let mut console = CONSOLE.lock();
            console.read_byte()
        };

        kprintln!("{}", byte);

        let val = (byte - 0x61) << 3;
        for i in 0..(framebuffer.buffer.len() / 3) {
            framebuffer.buffer[i * 3 + channel_counter].write(val);
        }
        channel_counter = (channel_counter + 1) % 3;
        kprintln!("{:?}", &framebuffer.buffer[0..3]);

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
