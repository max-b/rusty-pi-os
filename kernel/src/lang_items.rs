extern crate core;
extern crate std;

use std::alloc::Layout;

use lang_items::core::panic::PanicInfo;

#[lang = "eh_personality"] #[cfg(not(test))] pub extern fn eh_personality() {}

#[panic_handler] #[cfg(not(test))] #[no_mangle] pub extern fn panic_fmt(_panic: &PanicInfo) -> ! { loop{} }

#[cfg(not(test))]
#[doc(hidden)]
#[alloc_error_handler]
pub fn rust_oom(_layout: Layout) -> ! {
    loop{}
}
