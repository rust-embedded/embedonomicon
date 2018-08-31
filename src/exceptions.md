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

``` console
$ sed -n 57,92p ../rt/src/lib.rs
```

``` rust
{{#include ../ci/exceptions/rt/src/lib.rs:57:92}}
```

Some of the entries in the vector table are *reserved*; the ARM documentation states that they
should be assigned the value `0` so we use a union to do exactly that. The entries that must point
to a handler make use of *external* functions; this is important because it lets the end user
*provide* the actual function definition.

Next, we define a default exception handler in the Rust code. Exceptions that have not been assigned
a handler by the end user will make use of this default handler.

``` console
$ tail -n4 ../rt/src/lib.rs
```

``` rust
{{#include ../ci/exceptions/rt/src/lib.rs:94:97}}
```

## Linker script side

On the linker script side, we place these new exception vectors right after the reset vector.

``` console
$ sed -n 12,25p ../rt/link.x
```

``` text
{{#include ../ci/exceptions/rt/link.x:12:27}}
```

And we use `PROVIDE` to give a default value to the handlers that we left undefined in `rt` (`NMI`
and the others above):

``` console
$ tail -n8 ../rt/link.x
```

``` text
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
{{#include ../ci/exceptions/app/src/main.rs}}
```

``` console
(lldb) b DefaultExceptionHandler
Breakpoint 1: where = app`DefaultExceptionHandler at lib.rs:96, address = 0x000000ec

(lldb) continue
Process 1 resuming
Process 1 stopped
* thread #1, stop reason = breakpoint 1.1
    frame #0: 0x000000ec app`DefaultExceptionHandler at lib.rs:96
   93
   94   #[no_mangle]
   95   pub extern "C" fn DefaultExceptionHandler() {
-> 96       loop {}
   97   }
```

And for completeness, here's the disassembly of the optimized version of the program:

``` console
$ cargo objdump --bin app --release -- -d
```

> **NOTE** `llvm-objdump`, which is what `cargo-objdump` invokes, produces
> broken output for this particular file so the output below is actually the
> output from `arm-none-eabi-objdump`


``` text
00000040 <main>:
  40:   defe            udf     #254    ; 0xfe
  42:   defe            udf     #254    ; 0xfe

00000044 <Reset>:
  44:   f240 0100       movw    r1, #0
  48:   f240 0000       movw    r0, #0
  4c:   f2c2 0100       movt    r1, #8192       ; 0x2000
  50:   f2c2 0000       movt    r0, #8192       ; 0x2000
  54:   1a09            subs    r1, r1, r0
  56:   f000 f869       bl      12c <__aeabi_memclr>
  5a:   f240 0100       movw    r1, #0
  5e:   f240 0000       movw    r0, #0
  62:   f2c2 0100       movt    r1, #8192       ; 0x2000
  66:   f2c2 0000       movt    r0, #8192       ; 0x2000
  6a:   1a0a            subs    r2, r1, r0
  6c:   f240 1132       movw    r1, #306        ; 0x132
  70:   f2c0 0100       movt    r1, #0
  74:   f000 f804       bl      80 <__aeabi_memcpy>
  78:   f7ff ffe2       bl      40 <main>
  7c:   defe            udf     #254    ; 0xfe

0000007e <DefaultExceptionHandler>:
  7e:   e7fe            b.n     7e <DefaultExceptionHandler>
```

``` console
$ cargo objdump --bin app --release -- -s -j .vector_table
```

``` text
{{#include ../ci/exceptions/app/app.vector_table.objdump}}
```

The vector table now resembles the results of all the code snippets in this book
  so far. To summarize:
- In the [_Inspecting it_] section of the earlier memory chapter, we learned
  that:
    - The first entry in the vector table contains the initial value of the
      stack pointer.
    - Objdump prints in `little endian` format, so the stack starts at
      `0x2001_0000`.
    - The second entry points to address `0x0000_0045`, the Reset handler.
        - The address of the Reset handler can be seen in the disassembly above,
          being `0x44`.
        - The first bit being set to 1 does not alter the address due to
          alignment requirements. Instead, it causes the function to be executed
          in _thumb mode_.
- Afterwards, a pattern of addresses alternating between `0x7f` and `0x00` is
  visible.
    - Looking at the disassembly above, it is clear that `0x7f` refers to the
      `DefaultExceptionHandler` (`0x7e` executed in thumb mode).
    - Cross referencing the pattern to the vector table that was set up earlier
      in this chapter (see the definition of `pub static EXCEPTIONS`) with [the
      vector table layout for the Cortex-M], it is clear that the address of the
      `DefaultExceptionHandler` is present each time a respective handler entry
      is present in the table.
    - In turn, it is also visible that the layout of the vector table data
      structure in the Rust code is aligned with all the reserved slots in the
      Cortex-M vector table. Hence, all reserved slots are correctly set to a
      value of zero.

[_Inspecting it_]: https://rust-embedded.github.io/embedonomicon/memory-layout.html#inspecting-it
[the vector table layout for the Cortex-M]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/exception-model/vector-table

## Overriding a handler

To override an exception handler, the user has to provide a function whose symbol name exactly
matches the name we used in `EXCEPTIONS`.

``` rust
{{#include ../ci/exceptions/app2/src/main.rs}}
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
