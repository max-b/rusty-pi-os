#![feature(asm, lang_items)]
#![feature(panic_implementation)]
#![feature(panic_handler)]
#![feature(alloc_error_handler)]

extern crate pi;
extern crate xmodem;

pub mod lang_items;

use xmodem::{Xmodem, Progress};
use pi::uart::MiniUart;
use pi::timer::spin_sleep_ms;
use pi::console::*;
use pi::allocator::Allocator;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

/// Start address of the binary to load and of the bootloader.
const BOOTLOADER_START_ADDR: usize = 0x4000000;
const BINARY_START_ADDR: usize = 0x80000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
fn jump_to(addr: *mut u8) -> ! {
    kprintln!("We're jumping to {:?}", addr);
    unsafe {
        kprintln!("Contents of jump: {:x?}", *addr);
        asm!("br $0" : : "r"(addr as usize));
        loop { asm!("nop" :::: "volatile")  }
    }
}

pub fn progress_fn(_progress: Progress) {
}

#[no_mangle]
pub unsafe extern "C" fn kmain() {
    let mut uart = MiniUart::new();
    uart.set_read_timeout(750);

    let buffer = std::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE);

    kprintln!("Starting Bootloader");
    kprintln!("buffer size = {}", buffer.len());
    kprintln!("first few bytes of buffer = {:x},{:x},{:x},{:x},{:x},", buffer[0], buffer[1], buffer[2], buffer[3], buffer[4]);
    buffer[0] = 0xde;
    buffer[1] = 0xad;
    buffer[2] = 0xbe;
    buffer[3] = 0xef;
    kprintln!("first few bytes of buffer = {:x},{:x},{:x},{:x},{:x},", buffer[0], buffer[1], buffer[2], buffer[3], buffer[4]);
    kprintln!("Send file now...");
    loop {
        match Xmodem::receive_with_progress(&mut uart, &mut buffer[..], progress_fn) {
            Ok(num_bytes) => {
                loop {
                    spin_sleep_ms(5000);
                    kprintln!("Recieved binary of size {}, now jumping...", num_bytes);
                    kprintln!("Jumping to {:x}", BINARY_START_ADDR);
                    kprintln!("first few bytes of buffer = {:x},{:x},{:x},{:x},{:x},", buffer[0], buffer[1], buffer[2], buffer[3], buffer[4]);
                    jump_to(BINARY_START);
                }
            },
            Err(_err) => {
                continue;
            }
        }
    }
}
