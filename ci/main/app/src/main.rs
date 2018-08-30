#![no_std]
#![no_main]

extern crate rt;

#[no_mangle]
pub fn main() -> ! {
    let _x = 42;

    loop {}
}
