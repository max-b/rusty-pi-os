/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if !is_power_of_two(align) {
        panic!("align argument {} is not a power of 2.");
    }
    addr - (addr % align)
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    if addr % align == 0 {
        addr
    } else {
        align_down(addr + align, align)
    }
}

fn is_power_of_two(num: usize) -> bool {
    let mut test = num;
    while test & 1 == 0 && test > 1 {
        test = test >> 1;
    }
    test == 1
}
