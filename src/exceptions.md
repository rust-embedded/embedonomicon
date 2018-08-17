# Exception handling

During the "Memory layout" section, we decided to start out simple and leave out handling of
exceptions. In this section, we'll add support for handling them; this serves as an example of
how to achieve compile time overridable behavior in stable Rust (i.e. without relying on the
unstable `#[linkage = "weak"]` attribute, which makes a symbol weak).

## Background information

In a nutshell, *exceptions* are a mechanism the Cortex-M and other architectures provide to let
applications respond to asynchronous, usually external, events. The most prominent type of exception,
that most people will know, is the classical (hardware) interrupt.

The Cortex-M exception mechanism works like this:
When the processor receives a signal or event associated to a type of exception, it suspends
the execution of the current subroutine (by stashing the state in the call stack) and then proceeds
to execute the corresponding exception handler, another subroutine, in a new stack frame. After
finishing the execution of the exception handler (i.e. returning from it), the processor resumes the
execution of the suspended subroutine.

The processor uses the vector table to decide what handler to execute. Each entry in the table
contains a pointer to a handler, and each entry corresponds to a different exception type. For
example, the second entry is the reset handler, the third entry is the NMI (Non Maskable Interrupt)
handler, and so on.

As mentioned before, the processor expects the vector table to be at some specific location in memory,
and each entry in it can potentially be used by the processor at runtime. Hence, the entries must always
contain valid values. Furthermore, we want the `rt` crate to be flexible so the end user can customize the
behavior of each exception handler. Finally, the vector table resides in read only memory, or rather in not
easily modified memory, so the user has to register the handler statically, rather than at runtime.

To satisfy all these constraints, we'll assign a *default* value to all the entries of the vector
table in the `rt` crate, but make these values kind of *weak* to let the end user override them
at compile time.

## Rust side

Let's see how all this can be implemented. For simplicity, we'll only work with the first 16 entries
of the vector table; these entries are not device specific so they have the same function on any
kind of Cortex-M microcontroller.

The first thing we'll do is create an array of vectors (pointers to exception handlers) in the
`rt` crate's code:

``` rust
pub union Vector {
    reserved: u32,
    handler: unsafe extern "C" fn(),
}

extern "C" {
    fn NMI();
    fn HardFault();
    fn MemManage();
    fn BusFault();
    fn UsageFault();
    fn SVCall();
    fn PendSV();
    fn SysTick();
}

#[link_section = ".vector_table.exceptions"]
#[no_mangle]
pub static EXCEPTIONS: [Vector; 14] = [
    Vector { handler: NMI },
    Vector { handler: HardFault },
    Vector { handler: MemManage },
    Vector { handler: BusFault },
    Vector {
        handler: UsageFault,
    },
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
```

Some of the entries in the vector table are *reserved*; the ARM documentation states that they
should be assigned the value `0` so we use a union to do exactly that. The entries that must point
to a handler make use of *external* functions; this is important because it lets the end user
*provide* the actual function definition.

Next, we define a default exception handler in the Rust code. Exceptions that have not been assigned
a handler by the end user will make use of this default handler.

``` rust
#[no_mangle]
pub extern "C" fn DefaultExceptionHandler() {
    loop {}
}
```

## Linker script side

On the linker script side, we place these new exception vectors right after the reset vector.

```
EXTERN(RESET_VECTOR);
EXTERN(EXCEPTIONS); /* <- NEW */

SECTIONS
{
  .vector_table ORIGIN(FLASH) : ALIGN(4)
  {
    LONG(ORIGIN(RAM) + LENGTH(RAM));

    KEEP(*(.vector_table.reset_vector));

    KEEP(*(.vector_table.exceptions)); /* <- NEW */
  } > FLASH

  /* .. */
}
```

And we use `PROVIDE` to give a default value to the handlers that we left undefined in `rt` (`NMI`
and the others above):

```
PROVIDE(NMI = DefaultExceptionHandler);
PROVIDE(HardFault = DefaultExceptionHandler);
PROVIDE(MemManage = DefaultExceptionHandler);
PROVIDE(BusFault = DefaultExceptionHandler);
PROVIDE(UsageFault = DefaultExceptionHandler);
PROVIDE(SVCall = DefaultExceptionHandler);
PROVIDE(PendSV = DefaultExceptionHandler);
PROVIDE(SysTick = DefaultExceptionHandler);
```

`PROVIDE` only takes effect when the symbol to the left of the equal sign is still undefined after
inspecting all the input object files. This is the scenario where the user didn't implement the
handler for the respective exception.

## Testing it

That's it! The `rt` crate now has support for exception handlers. We can test it out with following
application:

``` rust
#![feature(core_intrinsics)]
#![no_main]
#![no_std]

#[macro_use]
extern crate rt;

use core::intrinsics;

entry!(main);

fn main() -> ! {
    // this executes the undefined instruction (UDF) and causes a HardFault exception
    unsafe { intrinsics::abort() }
}
```

``` console
(lldb) b DefaultExceptionHandler
Breakpoint 1: where = app`DefaultExceptionHandler at lib.rs:75, address = 0x000000e0

(lldb) continue
Process 1 resuming
Process 1 stopped
* thread #1, stop reason = breakpoint 1.1
    frame #0: 0x000000e0 app`DefaultExceptionHandler at lib.rs:75
   72
   73   #[no_mangle]
   74   pub extern "C" fn DefaultExceptionHandler() {
-> 75       loop {}
   76   }
   77
   78   #[no_mangle]
```

And for completeness, here's the disassembly of the optimized version of the program:

``` console
$ cargo objdump -- -d -no-show-raw-insn target/thumbv7m-none-eabi/release/app

target/thumbv7m-none-eabi/release/app:  file format ELF32-arm-little

Disassembly of section .text:
Reset:
      40:       movw    r1, #0
      44:       movw    r0, #0
      48:       movt    r1, #8192
      4c:       movt    r0, #8192
      50:       subs    r1, r1, r0
      52:       bl      #208
      56:       movw    r1, #0
      5a:       movw    r0, #0
      5e:       movt    r1, #8192
      62:       movt    r0, #8192
      66:       subs    r2, r1, r0
      68:       movw    r1, #0
      6c:       movt    r1, #0
      70:       bl      #6
      74:       trap
      76:       trap

DefaultExceptionHandler:
      78:       b       #-4 <DefaultExceptionHandler>

$ cargo objdump -- -s -j .vector_table target/thumbv7m-none-eabi/release/app

target/thumbv7m-none-eabi/release/app:  file format ELF32-arm-little

Contents of section .vector_table:
 0000 00000120 41000000 79000000 79000000  ... A...y...y...
 0010 79000000 79000000 79000000 00000000  y...y...y.......
 0020 00000000 00000000 00000000 79000000  ............y...
 0030 00000000 00000000 79000000 79000000  ........y...y...
```

The vector table now resembles the results of all the code snippets in this book so far. To summarize:
- In the [_Inspecting it_] section of the earlier memory chapter, we learned that:
    - The first entry in the vector table contains the initial value of the stack pointer.
    - Objdump prints in `little endian` format, so the stack starts at `0x2001_0000`.
    - The second entry points to address `0x0000_0041`, the Reset handler.
        - The address of the Reset handler can be seen in the disassembly above, being `0x40`.
        - The first bit being set to 1 does not alter the address due to alignment requirements. Instead, it causes the function to be executed in _thumb mode_.
- Afterwards, a pattern of addresses alternating between `0x79` and `0x00` is visible.
    - Looking at the disassembly above, it is clear that `0x79` refers to the `DefaultExceptionHandler` (`0x78` executed in thumb mode).
    - Cross referncing the pattern to the vector table that was set up earlier in this chapter (see the definition of `pub static EXCEPTIONS`) with [the vector table layout for the Cortex-M], it is clear that the address of the `DefaultExceptionHandler` is present each time a respective handler entry is present in the table.

[_Inspecting it_]: https://rust-embedded.github.io/embedonomicon/memory-layout.html#inspecting-it
[the vector table layout for the Cortex-M]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/exception-model/vector-table

## Overriding a handler

To override an exception handler, the user has to provide a function whose symbol name exactly
matches the name we used in `EXCEPTIONS`.

``` rust
#![feature(core_intrinsics)]
#![no_main]
#![no_std]

#[macro_use]
extern crate rt;

use core::intrinsics;

entry!(main);

fn main() -> ! {
    unsafe { intrinsics::abort() }
}

#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    // do something interesting here
    loop {}
}
```

You can test it in QEMU

``` console
(lldb) b HardFault

(lldb) continue
Process 1 resuming
Process 1 stopped
* thread #1, stop reason = breakpoint 1.1
    frame #0: 0x00000044 app`HardFault at main.rs:19
   16   #[no_mangle]
   17   pub extern "C" fn HardFault() -> ! {
   18       // do something interesting here
-> 19       loop {}
   20   }
```

The program now executes the user defined `HardFault` function instead of the
`DefaultExceptionHandler` in the `rt` crate.

Like our first attempt at a `main` interface, this first implementation has the problem of having no
type safety. It's also easy to mistype the name of the exception, but that doesn't produce an error
or warning. Instead the user defined handler is simply ignored. Those problems can be fixed using a
macro like the [`exception!`] macro defined in `cortex-m-rt`.

[`exception!`]: https://github.com/japaric/cortex-m-rt/blob/v0.5.1/src/lib.rs#L79
