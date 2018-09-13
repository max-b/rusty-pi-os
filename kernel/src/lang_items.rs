extern crate core;
extern crate std;

use pi::console::kprintln;
use std::alloc::Layout;

use lang_items::core::panic::PanicInfo;

#[lang = "eh_personality"] #[cfg(not(test))] pub extern fn eh_personality() {}

#[panic_handler]
#[cfg(not(test))]
#[no_mangle]
pub extern fn panic_fmt(panic_info: &PanicInfo) -> ! {
    kprintln!("
         )   (     (
        (    )     )
         )   (    (
        (     )     `
   .-''''^'''^^'''^''''-.
  (//\\//\\//\\//\\//\\//)
    ~^^^^^^^^^^^^^^^^^^/~
     `================`

  🥧  The pi is overdone 🥧🥧

😱---------- PANIC ----------😱");
    kprintln!("{:?}", &panic_info.payload());

    if let Some(location) = panic_info.location() {
        kprintln!("panic occurred in file '{}' at line {}", location.file(),
            location.line());
    } else {
        kprintln!("panic occurred but can't get location information...");
    }

    loop{}
}

#[cfg(not(test))]
#[doc(hidden)]
#[alloc_error_handler]
pub fn rust_oom(layout: Layout) -> ! {
    kprintln!("Out of Memory 😮");
    kprintln!("{:#?}", &layout);
    loop{}
}
