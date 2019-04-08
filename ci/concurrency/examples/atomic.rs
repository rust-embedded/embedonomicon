// source: examples/atomic.rs
#![no_main]
#![no_std]

extern crate panic_halt;

use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m_rt::{entry, exception};

static X: AtomicBool = AtomicBool::new(false);

#[entry]
fn main() -> ! {
    // omitted: configuring and enabling the `SysTick` interrupt

    // wait until `SysTick` returns before starting the main logic
    while !X.load(Ordering::Relaxed) {}

    loop {
        // main logic
    }
}

#[exception]
fn SysTick() {
    X.store(true, Ordering::Relaxed);
}
