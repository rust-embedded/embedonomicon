// source: examples/mutex.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use core::cell::{RefCell, UnsafeCell};

use bare_metal::CriticalSection;
use cortex_m::interrupt;
use cortex_m_rt::{entry, exception};

struct Mutex<T>(UnsafeCell<T>);

// TODO does T require a Sync / Send bound?
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    const fn new(value: T) -> Mutex<T> {
        Mutex(UnsafeCell::new(value))
    }

    // NOTE: the `'cs` constraint prevents the returned reference from outliving
    // the `CriticalSection` token
    fn borrow<'cs>(&self, _cs: &'cs CriticalSection) -> &'cs T {
        unsafe { &*self.0.get() }
    }
}

static COUNTER: Mutex<RefCell<u64>> = Mutex::new(RefCell::new(0));

#[inline(never)]
#[entry]
fn main() -> ! {
    loop {
        // `interrupt::free` runs the closure in a critical section (interrupts disabled)
        interrupt::free(|cs: &CriticalSection| {
            let counter: &RefCell<u64> = COUNTER.borrow(cs);

            *counter.borrow_mut() += 1;

            // &*counter.borrow() //~ ERROR: this reference cannot outlive the closure
        });
    }
}

#[exception]
fn SysTick() {
    interrupt::free(|cs| {
        let counter = COUNTER.borrow(cs);
        *counter.borrow_mut() *= 2;
    });
}
