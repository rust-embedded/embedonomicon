#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
#[inline(never)]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
