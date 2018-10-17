pub fn sleep(ms: u32) -> Result<(u32), String> {
    let error: u64;
    let actual_sleep_time: u32;
    unsafe {
        asm!("mov x0, $2
              svc 1
              mov $0, x0
              mov $1, x7"
              : "=r"(actual_sleep_time), "=r"(error)
              : "r"(ms)
              : "x0", "x7")
    }

    if error != 0 {
        Err(format!("Error in sleep syscall: {}", error))
    } else {
        Ok(actual_sleep_time)
    }
}
