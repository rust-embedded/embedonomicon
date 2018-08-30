#![no_std]
#![no_main]

#[macro_use]
extern crate rt;

entry!(main);

fn main() -> ! {
    let _x = 42;

    loop {}
}
