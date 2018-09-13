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
#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

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
pub mod draw;

use volatile::Writeable;
use pi::gpio::Gpio;
use pi::console::{CONSOLE, kprint, kprintln};
use racoon::RACOON_STRING;
use shell::shell;

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

    kprintln!("{}", RACCOON_STRING);

    for tag in pi::atags::Atags::get() {
        kprintln!("{:#?}", tag);
    }

    ALLOCATOR.initialize();
    let mut v = vec![];
    for i in 0..1000 {
        v.push(i);
    }
    kprintln!("allocated vec:");
    kprintln!("{:x?}", v);

    shell(">>> ");
}
