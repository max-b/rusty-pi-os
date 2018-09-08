#[lang = "eh_personality"] #[cfg(not(test))] pub extern fn eh_personality() {}

#[panic_implementation] #[cfg(not(test))] #[no_mangle] pub extern fn panic_fmt(_panic: &::core::panic::PanicInfo) -> ! { loop{} }

