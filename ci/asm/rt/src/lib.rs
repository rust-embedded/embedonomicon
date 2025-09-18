#![no_std]

use core::panic::PanicInfo;
// use core::ptr;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Reset() -> ! {
    // Omitted to simplify the `objdump` output
    // Initialize RAM
    unsafe extern "C" {
        // static mut _sbss: u8;
        // static mut _ebss: u8;

        // static mut _sdata: u8;
        // static mut _edata: u8;
        // static _sidata: u8;
    }

    // let count = &_ebss as *const u8 as usize - &_sbss as *const u8 as usize;
    // ptr::write_bytes(&mut _sbss as *mut u8, 0, count);

    // let count = &_edata as *const u8 as usize - &_sdata as *const u8 as usize;
    // ptr::copy_nonoverlapping(&_sidata as *const u8, &mut _sdata as *mut u8, count);

    // Call user entry point
    unsafe extern "Rust" {
        safe fn main() -> !;
    }

    main()
}

// The reset vector, a pointer into the reset handler
#[unsafe(link_section = ".vector_table.reset_vector")]
#[unsafe(no_mangle)]
pub static RESET_VECTOR: unsafe extern "C" fn() -> ! = Reset;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[unsafe(export_name = "main")]
        pub unsafe fn __main() -> ! {
            // type check the given path
            let f: fn() -> ! = $path;

            f()
        }
    };
}

pub union Vector {
    reserved: u32,
    handler: unsafe extern "C" fn(),
}

unsafe extern "C" {
    fn NMI();
    fn HardFaultTrampoline(); // <- CHANGED!
    fn MemManage();
    fn BusFault();
    fn UsageFault();
    fn SVCall();
    fn PendSV();
    fn SysTick();
}

#[unsafe(link_section = ".vector_table.exceptions")]
#[unsafe(no_mangle)]
pub static EXCEPTIONS: [Vector; 14] = [
    Vector { handler: NMI },
    Vector { handler: HardFaultTrampoline }, // <- CHANGED!
    Vector { handler: MemManage },
    Vector { handler: BusFault },
    Vector { handler: UsageFault },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: SVCall },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: PendSV },
    Vector { handler: SysTick },
];

#[unsafe(no_mangle)]
pub extern "C" fn DefaultExceptionHandler() {
    loop {}
}
