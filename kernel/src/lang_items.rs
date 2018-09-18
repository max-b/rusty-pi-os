extern crate core;
extern crate std;

use pi::console::kprintln;
use pi::screen::SCREEN;
use std::alloc::Layout;

use lang_items::core::panic::PanicInfo;

#[lang = "eh_personality"]
#[cfg(not(test))]
pub extern "C" fn eh_personality() {}

pub const OVERDONE_STRING: &str = "
         )   (     (
        (    )     )
         )   (    (
        (     )     `
   .-''''^'''^^'''^''''-.
  (//\\//\\//\\//\\//\\//)
    ~^^^^^^^^^^^^^^^^^^/~
     `================`

  🥧  The pi is overdone 🥧🥧

😱---------- PANIC ----------😱";

#[panic_handler]
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn panic_fmt(panic_info: &PanicInfo) -> ! {

    kprintln!("{}", OVERDONE_STRING);
    kprintln!("{:?}", &panic_info.payload());

    if let Some(location) = panic_info.location() {
        kprintln!(
            "panic occurred in file '{}' at line {}",
            location.file(),
            location.line()
        );
    } else {
        kprintln!("panic occurred but can't get location information...");
    }

    SCREEN.lock().draw_string(&OVERDONE_STRING);
    SCREEN.lock().draw_char(0x0d);
    loop {}
}

#[cfg(not(test))]
#[doc(hidden)]
#[alloc_error_handler]
pub fn rust_oom(layout: Layout) -> ! {
    kprintln!("Out of Memory 😮");
    kprintln!("{:#?}", &layout);
    loop {}
}
