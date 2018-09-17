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
#![feature(naked_functions)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate fat32;
extern crate pi;
extern crate stack_vec;
extern crate volatile;

pub mod draw;
pub mod fs;
pub mod lang_items;
pub mod shell;
pub mod traps;
pub mod aarch64;
pub mod process;
pub mod vm;

use pi::console::kprintln;
use pi::raccoon::RACCOON_STRING;
use pi::timer;
use shell::shell;

#[cfg(not(test))]
use pi::allocator::Allocator;

use fs::FileSystem;
use process::GlobalScheduler;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();

#[no_mangle]
#[cfg(not(test))]
pub unsafe extern "C" fn kmain() {
    timer::spin_sleep_ms(1000);

    kprintln!("{}", RACCOON_STRING);

    for tag in pi::atags::Atags::get() {
        kprintln!("{:#?}", tag);
    }

    ALLOCATOR.initialize();
    FILE_SYSTEM.initialize();

    let mut v = vec![];
    for i in 0..1000 {
        v.push(i);
    }
    kprintln!("allocated vec:");
    kprintln!("{:x?}", v);

    shell(">>> ");
}
