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
use pi::screen::SCREEN;
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

    let el = aarch64::current_el();
    kprintln!("running in el {}", el);

    ALLOCATOR.initialize();
    FILE_SYSTEM.initialize();

    let mut v = vec![];
    for i in 0..1000 {
        v.push(i);
    }
    kprintln!("allocated vec:");
    kprintln!("{:x?}", v);

    SCREEN.lock().clear();
    SCREEN.lock().draw_string_scale(&"WELCOME TO MaxOS,5", 5);
    SCREEN.lock().draw_char_scale(0x0d, 5);

    SCHEDULER.start();
}

#[no_mangle]
pub extern fn start_shell() {
    // shell("!>>> ");
    // unsafe { asm!("brk 1" :::: "volatile"); }
    // unsafe { asm!("brk 2" :::: "volatile"); }
    // unsafe { asm!("brk 3" :::: "volatile"); }
    loop { shell("1 >>> "); }
}

#[no_mangle]
pub extern fn start_shell_2() {
    loop { shell("2 >>> "); }
}


#[no_mangle]
pub extern fn print_junk_1() {
    let mut counter = 0;
    loop {
        counter += 1;
        kprintln!("counter = {}", counter);
        timer::spin_sleep_ms(1000);
    }
}
