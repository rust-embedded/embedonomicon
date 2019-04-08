// source: examples/cs2.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use core::sync::atomic::{self, Ordering};

use cortex_m_rt::{entry, exception};

static mut COUNTER: u64 = 0;

#[inline(never)]
#[entry]
fn main() -> ! {
    let mut syst = cortex_m::Peripherals::take().unwrap().SYST;

    // omitted: configuring and enabling the `SysTick` interrupt

    loop {
        // `SysTick` can preempt `main` at this point

        // start of critical section: disable the `SysTick` interrupt
        syst.disable_interrupt();
        // ^ this method is implemented as shown in the comment below
        //
        // ```
        // let csr = ptr::read_volatile(0xE000_E010);`
        // ptr::write_volatile(0xE000_E010, csr & !(1 << 1));
        // ```

        // a compiler barrier equivalent to the "memory" clobber
        atomic::compiler_fence(Ordering::SeqCst);

        // `SysTick` can not preempt this block
        {
            let counter: &mut u64 = unsafe { &mut COUNTER };

            *counter += 1;
        }

        atomic::compiler_fence(Ordering::SeqCst);

        // end of critical section: re-enable the `SysTick` interrupt
        syst.enable_interrupt();
        // ^ this method is implemented as shown in the comment below
        //
        // ```
        // let csr = ptr::read_volatile(0xE000_E010);`
        // ptr::write_volatile(0xE000_E010, csr | (1 << 1));
        // ```

        // `SysTick` can start at this point
    }
}

#[exception]
fn SysTick() {
    // exclusive access to `COUNTER`
    let counter: &mut u64 = unsafe { &mut COUNTER };

    *counter += 1;
}
