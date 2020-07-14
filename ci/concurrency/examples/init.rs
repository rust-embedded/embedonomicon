// source: examples/init.rs

#![feature(maybe_uninit)]
#![no_main]
#![no_std]

extern crate panic_halt;

use core::mem::MaybeUninit;

use cortex_m::{asm, interrupt};
use cortex_m_rt::{entry, exception};

struct Thing {
    _state: (),
}

impl Thing {
    // NOTE the constructor is not `const`
    fn new() -> Self {
        Thing { _state: () }
    }

    fn do_stuff(&mut self) {
        // ..
    }
}

// uninitialized static variable
static mut THING: MaybeUninit<Thing> = MaybeUninit::uninitialized();

#[entry]
fn main() -> ! {
    // # Initialization phase

    // done as soon as the device boots
    interrupt::disable();

    // critical section that can't be preempted by any interrupt
    {
        // initialize the static variable at runtime
        unsafe { THING.set(Thing::new()) };

        // omitted: configuring and enabling the `SysTick` interrupt
    }

    // reminder: this is a compiler barrier
    unsafe { interrupt::enable() }

    // # main loop

    // `SysTick` can preempt `main` at this point

    loop {
        asm::nop();
    }
}

#[exception]
fn SysTick() {
    // this handler always observes the variable as initialized
    let thing: &mut Thing = unsafe { &mut *THING.as_mut_ptr() };

    thing.do_stuff();
}
