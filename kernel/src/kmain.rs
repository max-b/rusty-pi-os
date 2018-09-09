#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(never_type)]
#![feature(ptr_internals)]
#![feature(panic_implementation)]
#![feature(panic_handler)]
#![feature(nll)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api, global_allocator)]
#![feature(alloc_error_handler)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate pi;
extern crate stack_vec;
extern crate fat32;
extern crate volatile;

pub mod lang_items;
pub mod shell;
pub mod racoon;
pub mod fs;

use volatile::Writeable;
use pi::gpio::Gpio;
use pi::console::{CONSOLE, kprint, kprintln};
use pi::framebuffer::{Framebuffer, Pixel};
use racoon::RACOON_STRING;

#[cfg(not(test))]
use pi::allocator::Allocator;

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

    let mut framebuffer = Framebuffer::new().expect("Error creating new framebuffer");

    let mut pixel_cursor: Pixel = Default::default();

    loop {

        kprint!("<- ");

        let byte = {
            let mut console = CONSOLE.lock();
            console.read_byte()
        };

        kprintln!("0x{:x}", byte);

        if byte == 0x61 {
            pixel_cursor.x = pixel_cursor.x.wrapping_sub(1);
        }
        if byte == 0x64 {
            pixel_cursor.x = pixel_cursor.x.wrapping_add(1);
        }
        if byte == 0x73 {
            pixel_cursor.y = pixel_cursor.y.wrapping_add(1);
        }
        if byte == 0x77 {
            pixel_cursor.y = pixel_cursor.y.wrapping_sub(1);
        }
        if byte == 0x31 {
            pixel_cursor.color.red = pixel_cursor.color.red.wrapping_add(10);
        }
        if byte == 0x32 {
            pixel_cursor.color.green = pixel_cursor.color.green.wrapping_add(10);
        }
        if byte == 0x33 {
            pixel_cursor.color.blue = pixel_cursor.color.blue.wrapping_add(10);
        }

        framebuffer.draw_pixel(&pixel_cursor);

        if byte == 0x20 {
            framebuffer.clear();
        }

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
