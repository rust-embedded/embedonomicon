// source: examples/state.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::asm;
use cortex_m_rt::{entry, exception};

#[inline(never)]
#[entry]
fn main() -> ! {
    loop {
        // SysTick(); //~ ERROR: cannot find function `SysTick` in this scope

        asm::nop();
    }
}

#[exception]
fn SysTick() {
    static mut COUNTER: u64 = 0;

    // user code
    *COUNTER += 1;

    // SysTick(); //~ ERROR: cannot find function `SysTick` in this scope
}
