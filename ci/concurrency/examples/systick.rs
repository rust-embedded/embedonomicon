// source: examples/systick.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{asm, peripheral::syst::SystClkSource, Peripherals};
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::hprint;

// program entry point
#[entry]
fn main() -> ! {
    let mut syst = Peripherals::take().unwrap().SYST;

    // configures the system timer to trigger a SysTick interrupt every second
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(12_000_000); // period = 1s
    syst.enable_counter();
    syst.enable_interrupt();

    loop {
        asm::nop();
    }
}

// interrupt handler
// NOTE: the function name must match the name of the interrupt
#[exception]
fn SysTick() {
    hprint!(".").unwrap();
}
