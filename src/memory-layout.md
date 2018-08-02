# Memory layout

The next step is to ensure the program has the right memory layout so that the target system will be
able to execute it. In our example, we'll be working with a virtual Cortex-M3 microcontroller: the
[LM3S6965]. Our program will be the only process running on the device so it must also take care of
initializing the device.

## Background information

[LM3S6965]: http://www.ti.com/product/LM3S6965

Cortex-M devices require a [vector table] to be present at the start of their [code memory region].
The vector table is an array of pointers; the first two pointers are required to boot the device;
the rest of pointers are related to exceptions -- we'll ignore them for now.

[code memory region]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/memory-model
[vector table]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/exception-model/vector-table

Linkers decide the final memory layout of programs, but we can use [linker scripts] to have some
control over the memory layout. The control granularity that linker scripts give us over the layout
is at the level of *sections*. A section is a collection of *symbols* laid out in contiguous memory.
A symbol can be a statically allocated variable, a `static` variable, or a set of instructions, a
monomorphized (non generic) Rust function.

[linker scripts]: https://sourceware.org/binutils/docs/ld/Scripts.html

Every symbol has a name assigned by the compiler. As of Rust 1.28 , the Rust compiler assigns to
symbols names of the form: `_ZN5krate6module8function17he1dfc17c86fe16daE`, which demangles to
`krate::module::function::he1dfc17c86fe16da` where `krate::module::function` is the path of the
function or variable and `he1dfc17c86fe16da` is some sort of hash. The Rust compiler will place each
symbol into its own and unique section; for example the symbol mentioned before will be placed in a
section named `.text._ZN5krate6module8function17he1dfc17c86fe16daE`.

These compiler generated symbol and section names are not guaranteed to remain constant across
different releases of the Rust compiler. However, the language lets us control symbol names and
section placement via these attributes:

- `#[export_name = "foo"]` sets the symbol name to `foo`.
- `#[no_mangle]` means: use the function or variable name (not its full path) as its symbol name.
  `#[no_mangle] fn bar()` will produce a symbol named `bar`.
- `#[link_section = ".bar"]` places the symbol in a section named `.bar`.

With these attributes we can expose a stable ABI from the program and use it in the linker script.

## The Rust side

We need to populate the first two entries of the vector table. The first one, the initial value for
the stack pointer, can be populated using only the linker script. The second one, the reset vector,
needs to be created in Rust code and placed in the right place using the linker script.

The reset vector is a pointer into the reset handler. The reset handler is the function that the
device will execute after a system reset, or after it powers up for the first time. The reset
handler is always the first stack frame in the hardware call stack; returning from it is undefined
behavior as there's no other stack frame to return to. We can enforce that the reset handler never
returns by making it a divergent function, a function with signature `fn(/* .. */) -> !`.

``` rust
// The reset handler
#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    let x = 42;

    // can't return so we go into an infinite loop here
    loop {}
}

// The reset vector, a pointer into the reset handler
#[link_section = ".vector_table.reset_vector"]
#[no_mangle]
pub static RESET_VECTOR: unsafe extern "C" fn() -> ! = Reset;
```

We use `extern "C"` to tell the compiler to lower the function using the C ABI instead of the Rust
ABI, which is unstable, as that's what the hardware expects.

To refer to the reset handler and reset vector from the linker script we need them to have a stable
symbol name so we use `#[no_mangle]`. We need fine control over the location of `RESET_VECTOR` so we
place it on a known section, `.vector_table.reset_vector`. The exact location of the reset handler
itself, `Reset`, is not important so we just stick to the default compiler generated section.

Also, the linker will ignore symbols with internal linkage, AKA internal symbols, while traversing
the list of input object files so we need our two symbols to have external linkage. The only way to
make a symbol external in Rust is to make its corresponding item public (`pub`) and *reachable* (no
private module between the item and the root of the crate).

## The linker script side

Below is shown a minimal linker script that places the vector table in the right location. Let's
walk through it.

``` console
$ cat link.x

/* Memory layout of the LM3S6965 microcontroller */
/* 1K = 1 KiBi = 1024 bytes */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}

/* The entry point is the reset handler */
ENTRY(Reset);

EXTERN(RESET_VECTOR);

SECTIONS
{
  .vector_table ORIGIN(FLASH) :
  {
    /* First entry: initial Stack Pointer value */
    LONG(ORIGIN(RAM) + LENGTH(RAM));

    /* Second entry: reset vector */
    KEEP(*(.vector_table.reset_vector));
  } > FLASH

  .text :
  {
    *(.text .text.*);
  } > FLASH

  /DISCARD/ :
  {
    *(.ARM.exidx.*);
  }
}
```

### `MEMORY`

This section of the linker script describes the location and size of blocks of memory in the target.
Two memory blocks are defined: `FLASH` and `RAM`; they correspond to the physical memory available
in the target. The values used here correspond to the LM3S6965 microcontroller.

### `ENTRY`

Here we indicate to the linker that the reset handler -- whose symbol name is `Reset` -- is the
*entry point* of the program. Linkers aggressively discard unused sections. Linkers consider the
entry point and functions called from it as *used* so they won't discard them. Without this line the
linker would discard the `Reset` function and all other functions called from it.

### `EXTERN`

Linkers are lazy; they will stop looking into the input object files once they have found all the
symbols recursively referenced from the entry point. `EXTERN` forces the linker to look for its
argument even after all other referenced symbol have been found. As a rule of thumb, if you need a
symbol that's not called from the entry point to always be present in the output binary you should
use `EXTERN` in conjunction with `KEEP`.

### `SECTIONS`

This part describes how sections in the input object files, AKA *input sections*, are to be arranged
in the sections the output object file, AKA output sections; or if they should be discarded. Here we
define two output sections:

```
  .vector_table ORIGIN(FLASH) : { /* .. */ } > FLASH
```

`.vector_table`, which contains the vector table and its located at the start of `FLASH` memory;

```
  .text : { /* .. */ } > FLASH
```

and `.text`, which contains the program subroutines and its located somewhere in `FLASH`. Its start
address is not specified but the linker will place after the previous output section,
`.vector_table`.

The output `.vector_table` section contains:

```
    /* First entry: initial Stack Pointer value */
    LONG(ORIGIN(RAM) + LENGTH(RAM));
```

We'll place the (call) stack at the end of RAM (the stack is *full descending*; it grows towards
smaller addresses) so the end address of RAM will be used as the initial Stack Pointer (SP) value.
That address is computed in the linker script itself using the information we entered for the `RAM`
memory block.

```
    /* Second entry: reset vector */
    KEEP(*(.vector_table.reset_vector));
```

Next, we use `KEEP` to force the linker to insert all input sections named
`.vector_table.reset_vector` right after the initial SP value. The only symbol located in that
section is `RESET_VECTOR` so this will effectively place `RESET_VECTOR` second in the vector table.

The output `.text` section contains:

```
    *(.text .text.*);
```

All the input sections named `.text` and `.text.*`. Note that we don't use `KEEP` here to let the
linker discard the unused sections.

Finally, we use the special `/DISCARD/` section to discard:

```
    *(.ARM.exidx.*);
```

input sections named `.ARM.exidx.*`. These sections are related to exception handling but we are not
doing stack unwinding on panics and they take up space in Flash memory so we just discard them.

## Putting it all together

Now we can link the application. For reference, here's the complete Rust program:

``` rust
#![feature(panic_implementation)]
#![no_main]
#![no_std]

use core::panic::PanicInfo;

// The reset handler
#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    loop {}
}

// The reset vector.
#[link_section = ".vector_table.reset_vector"]
#[no_mangle]
pub static RESET_VECTOR: unsafe extern "C" fn() -> ! = Reset;

#[no_mangle]
#[panic_implementation]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}
```

We'll use the LLVM linker, LLD, shipped with the Rust toolchain. That way you won't need to install
the `arm-none-eabi-gcc` linker that the `thumbv7m-none-eabi` target uses by default. Changing the
linker is done via rustc flags; the full Cargo invocation to change the linker and pass the linker
script to the linker is shown below:

``` console
$ cargo rustc -- \
      -C linker=rust-lld \
      -Z linker-flavor=ld.lld \
      -C link-arg=-Tlink.x
```

## Inspecting it

Now let's inspect the output binary to confirm the memory layout looks the way we want:

``` console
$ cargo objdump -- -d -no-show-raw-insn target/thumbv7m-none-eabi/debug/app

target/thumbv7m-none-eabi/debug/app:    file format ELF32-arm-little

Disassembly of section .text:
Reset:
       8:       sub     sp, #4
       a:       movs    r0, #42
       c:       str     r0, [sp]
       e:       b       #-2 <Reset+0x8>
      10:       b       #-4 <Reset+0x8>
```

This is the disassembly of the `.text` section. We see that the reset handler, named `Reset`, is
located at address `0x8`.

``` console
$ cargo objdump -- -s -j .vector_table target/thumbv7m-none-eabi/debug/app

target/thumbv7m-none-eabi/debug/app:    file format ELF32-arm-little

Contents of section .vector_table:
 0000 00000120 09000000                    ... ....
```

This shows the contents of the `.vector_table` section. We can see that the section starts at
address `0x0` and that the first word of the section is `0x2001_0000` (the `objdump` output is in
little endian format). This is the initial SP value and matches the end address of RAM. The second
word is `0x9`; this is the *thumb mode* address of the reset handler. When a function is to be
executed in thumb mode the first bit of its address is set to 1.

## Testing it

This program is a valid LM3S6965 program; we can execute it in a virtual microcontroller (QEMU) to
test it out.

``` console
$ # this program will block
$ qemu-system-arm \
      -cpu cortex-m3 \
      -machine lm3s6965evb \
      -gdb tcp::3333 \
      -S \
      -nographic \
      -kernel target/thumbv7m-none-eabi/debug/app
```

``` console
$ # on a different terminal
$ lldb target/thumbv7m-none-eabi/debug/app

(lldb) gdb-remote 3333
Process 1 stopped
* thread #1, stop reason = signal SIGTRAP
    frame #0: 0x00000008 app`Reset at main.rs:23
   20   #[no_mangle]
   21   #[panic_implementation]
   22   fn panic(_panic: &PanicInfo) -> ! {
-> 23       loop {}
   24   }

(lldb) # ^ that source is wrong; the processor is about to execute Reset
(lldb) disassemble -frame
app`Reset:
->  0x8 <+0>:  sub    sp, #0x4
    0xa <+2>:  movs   r0, #0x2a
    0xc <+4>:  str    r0, [sp]
    0xe <+6>:  b      0x10                      ; <+8> at main.rs:12
    0x10 <+8>: b      0x10                      ; <+8> at main.rs:12

(lldb) # the SP has the initial value we programmed in the vector table
(lldb) print/x $sp
(unsigned int) $0 = 0x20010000

(lldb) step
(lldb) step
(lldb) step
Process 1 stopped
* thread #1, stop reason = step in
    frame #0: 0x0000000e app`Reset at main.rs:12
   9    pub unsafe extern "C" fn Reset() -> ! {
   10       let x = 42;
   11
-> 12       loop {}
   13   }
   14
   15   // The reset vector.

(lldb) # next we inspect the stack variable `x`
(lldb) print x
(int) $1 = 42

(lldb) print &x
(int *) $2 = 0x2000fffc

(lldb) exit
```
