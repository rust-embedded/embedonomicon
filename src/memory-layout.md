# Memory layout

The next step is to ensure the program has the right memory layout so that the target system will be
able to execute it. In our example, we'll be working with a virtual Cortex-M3 microcontroller: the
[LM3S6965]. Our program will be the only process running on the device so it must also take care of
initializing the device.

## Background information

[LM3S6965]: http://www.ti.com/product/LM3S6965

Cortex-M devices require a [vector table] to be present at the start of their [code memory region].
The vector table is an array of pointers; the first two pointers are required to boot the device,
the rest of the pointers are related to exceptions. We'll ignore them for now.

[code memory region]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/memory-model
[vector table]: https://developer.arm.com/docs/dui0552/latest/the-cortex-m3-processor/exception-model/vector-table

Linkers decide the final memory layout of programs, but we can use [linker scripts] to have some
control over it. The control granularity that linker scripts give us over the layout
is at the level of *sections*. A section is a collection of *symbols* laid out in contiguous memory.
Symbols, in turn, can be data (a static variable), or instructions (a Rust function).

[linker scripts]: https://sourceware.org/binutils/docs/ld/Scripts.html

Every symbol has a name assigned by the compiler. As of Rust 1.28 , the names that the Rust compiler
assigns to symbols are of the form: `_ZN5krate6module8function17he1dfc17c86fe16daE`, which demangles to
`krate::module::function::he1dfc17c86fe16da` where `krate::module::function` is the path of the
function or variable and `he1dfc17c86fe16da` is some sort of hash. The Rust compiler will place each
symbol into its own unique section; for example the symbol mentioned before will be placed in a
section named `.text._ZN5krate6module8function17he1dfc17c86fe16daE`.

These compiler generated symbol and section names are not guaranteed to remain constant across
different releases of the Rust compiler. However, the language lets us control symbol names and
section placement via these attributes:

- `#[export_name = "foo"]` sets the symbol name to `foo`.
- `#[no_mangle]` means: use the function or variable name (not its full path) as its symbol name.
  `#[no_mangle] fn bar()` will produce a symbol named `bar`.
- `#[link_section = ".bar"]` places the symbol in a section named `.bar`.

With these attributes, we can expose a stable ABI of the program and use it in the linker script.

## The Rust side

As mentioned above, for Cortex-M devices, we need to populate the first two entries of the
vector table. The first one, the initial value for the stack pointer, can be populated using
only the linker script. The second one, the reset vector, needs to be created in Rust code
and placed correctly using the linker script.

The reset vector is a pointer into the reset handler. The reset handler is the function that the
device will execute after a system reset, or after it powers up for the first time. The reset
handler is always the first stack frame in the hardware call stack; returning from it is undefined
behavior as there's no other stack frame to return to. We can enforce that the reset handler never
returns by making it a divergent function, which is a function with signature `fn(/* .. */) -> !`.

``` rust
{{#include ../ci/memory-layout/src/main.rs:7:19}}
```

The hardware expects a certain format here, to which we adhere by using `extern "C"` to tell the
compiler to lower the function using the C ABI, instead of the Rust ABI, which is unstable.

To refer to the reset handler and reset vector from the linker script, we need them to have a stable
symbol name so we use `#[no_mangle]`. We need fine control over the location of `RESET_VECTOR`, so we
place it in a known section, `.vector_table.reset_vector`. The exact location of the reset handler
itself, `Reset`, is not important. We just stick to the default compiler generated section.

The linker will ignore symbols with internal linkage (also known as internal symbols) while traversing
the list of input object files, so we need our two symbols to have external linkage. The only way to
make a symbol external in Rust is to make its corresponding item public (`pub`) and *reachable* (no
private module between the item and the root of the crate).

## The linker script side

A minimal linker script that places the vector table in the correct location is shown below. Let's
walk through it.

``` console
$ cat link.x
```

``` text
{{#include ../ci/memory-layout/link.x}}
```

### `MEMORY`

This section of the linker script describes the location and size of blocks of memory in the target.
Two memory blocks are defined: `FLASH` and `RAM`; they correspond to the physical memory available
in the target. The values used here correspond to the LM3S6965 microcontroller.

### `ENTRY`

Here we indicate to the linker that the reset handler, whose symbol name is `Reset`, is the
*entry point* of the program. Linkers aggressively discard unused sections. Linkers consider the
entry point and functions called from it as *used* so they won't discard them. Without this line,
the linker would discard the `Reset` function and all subsequent functions called from it.

### `EXTERN`

Linkers are lazy; they will stop looking into the input object files once they have found all the
symbols that are recursively referenced from the entry point. `EXTERN` forces the linker to look
for `EXTERN`'s argument even after all other referenced symbols have been found. As a rule of thumb,
if you need a symbol that's not called from the entry point to always be present in the output binary,
you should use `EXTERN` in conjunction with `KEEP`.

### `SECTIONS`

This part describes how sections in the input object files (also known as *input sections*) are to be arranged
in the sections of the output object file (also known as output sections) or if they should be discarded. Here
we define two output sections:

``` text
  .vector_table ORIGIN(FLASH) : { /* .. */ } > FLASH
```

`.vector_table` contains the vector table and is located at the start of `FLASH` memory.

``` text
  .text : { /* .. */ } > FLASH
```

`.text` contains the program subroutines and is located somewhere in `FLASH`. Its start
address is not specified, but the linker will place it after the previous output section,
`.vector_table`.

The output `.vector_table` section contains:

``` text
{{#include ../ci/memory-layout/link.x:18:19}}
```

We'll place the (call) stack at the end of RAM (the stack is *full descending*; it grows towards
smaller addresses) so the end address of RAM will be used as the initial Stack Pointer (SP) value.
That address is computed in the linker script itself using the information we entered for the `RAM`
memory block.

```
{{#include ../ci/memory-layout/link.x:21:22}}
```

Next, we use `KEEP` to force the linker to insert all input sections named
`.vector_table.reset_vector` right after the initial SP value. The only symbol located in that
section is `RESET_VECTOR`, so this will effectively place `RESET_VECTOR` second in the vector table.

The output `.text` section contains:

``` text
{{#include ../ci/memory-layout/link.x:27}}
```

This includes all the input sections named `.text` and `.text.*`. Note that we don't use `KEEP`
here to let the linker discard unused sections.

Finally, we use the special `/DISCARD/` section to discard

``` text
{{#include ../ci/memory-layout/link.x:32}}
```

input sections named `.ARM.exidx.*`. These sections are related to exception handling but we are not
doing stack unwinding on panics and they take up space in Flash memory, so we just discard them.

## Putting it all together

Now we can link the application. For reference, here's the complete Rust program:

``` rust
{{#include ../ci/memory-layout/src/main.rs}}
```

We have to tweak the linker process to make it use our linker script. This is done
passing the `-C link-arg` flag to `rustc`. This can be done with `cargo-rustc` or
`cargo-build`.

**IMPORTANT**: Make sure you have the `.cargo/config` file that was added at the
end of the last section before running this command.

Using the `cargo-rustc` subcommand:

``` console
$ cargo rustc -- -C link-arg=-Tlink.x
```

Or you can set the rustflags in `.cargo/config` and continue using the
`cargo-build` subcommand. We'll do the latter because it better integrates with
`cargo-binutils`.

``` console
# modify .cargo/config so it has these contents
$ cat .cargo/config
```

``` toml
{{#include ../ci/memory-layout/.cargo/config}}
```

The `[target.thumbv7m-none-eabi]` part says that these flags will only be used
when cross compiling to that target.

## Inspecting it

Now let's inspect the output binary to confirm the memory layout looks the way we want
(this requires [`cargo-binutils`](https://github.com/rust-embedded/cargo-binutils#readme)):

``` console
$ cargo objdump --bin app -- -d -no-show-raw-insn
```

``` text
{{#include ../ci/memory-layout/app.text.objdump}}
```

This is the disassembly of the `.text` section. We see that the reset handler, named `Reset`, is
located at address `0x8`.

``` console
$ cargo objdump --bin app -- -s --section .vector_table
```

``` text
{{#include ../ci/memory-layout/app.vector_table.objdump}}
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
$ arm-none-eabi-gdb -q target/thumbv7m-none-eabi/debug/app
Reading symbols from target/thumbv7m-none-eabi/debug/app...done.

(gdb) target remote :3333
Remote debugging using :3333
Reset () at src/main.rs:8
8       pub unsafe extern "C" fn Reset() -> ! {

(gdb) # the SP has the initial value we programmed in the vector table
(gdb) print/x $sp
$1 = 0x20010000

(gdb) step
9           let _x = 42;

(gdb) step
12          loop {}

(gdb) # next we inspect the stack variable `_x`
(gdb) print _x
$2 = 42

(gdb) print &_x
$3 = (i32 *) 0x2000fffc

(gdb) quit
```
