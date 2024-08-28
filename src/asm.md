# Assembly on stable

> Note: Since Rust 1.59, both *inline* assembly (`asm!`) and *free form* assembly
> (`global_asm!`) become stable. But since it will take some time for the 
> existing crates to catchup the change, and since it's good for us to know the
> other ways in history we used to deal with assembly, we will keep this chapter
> here.

So far we have managed to boot the device and handle interrupts without a single
line of assembly. That's quite a feat! But depending on the architecture you are
targeting you may need some assembly to get to this point. There are also some
operations, for example context switching, that require assembly.

The problem is that both *inline* assembly (`asm!`) and *free form* assembly
(`global_asm!`) are unstable, and there's no estimate for when they'll be
stabilized, so you can't use them on stable . This is not a showstopper because
there are some workarounds which we'll document here.

To motivate this section we'll tweak the `HardFault` handler to provide
information about the stack frame that generated the exception.

Here's what we want to do:

Instead of letting the user directly put their `HardFault` handler in the vector
table we'll make the `rt` crate put a trampoline to the user-defined `HardFault`
handler in the vector table.

``` console
$ tail -n36 ../rt/src/lib.rs
```

``` rust
{{#include ../ci/asm/rt/src/lib.rs:61:96}}
```

This trampoline will read the stack pointer and then call the user `HardFault`
handler. The trampoline will have to be written in assembly:

``` armasm
{{#include ../ci/asm/rt/asm.s:5:6}}
```

Due to how the ARM ABI works this sets the Main Stack Pointer (MSP) as the first
argument of the `HardFault` function / routine. This MSP value also happens to
be a pointer to the registers pushed to the stack by the exception. With these
changes the user `HardFault` handler must now have signature
`fn(&StackedRegisters) -> !`.

## `.s` files

One approach to stable assembly is to write the assembly in an external file:

``` console
$ cat ../rt/asm.s
```

``` armasm
{{#include ../ci/asm/rt/asm.s}}
```

And use the `cc` crate in the build script of the `rt` crate to assemble that
file into an object file (`.o`) and then into an archive (`.a`).

``` console
$ cat ../rt/build.rs
```

``` rust
{{#include ../ci/asm/rt/build.rs}}
```

``` console
$ tail -n2 ../rt/Cargo.toml
```

``` toml
{{#include ../ci/asm/rt/Cargo.toml:7:8}}
```

And that's it!

We can confirm that the vector table contains a pointer to `HardFaultTrampoline`
by writing a very simple program.

``` rust
{{#include ../ci/asm/app/src/main.rs}}
```

Here's the disassembly. Look at the address of `HardFaultTrampoline`.

``` console
$ cargo objdump --bin app --release -- -d --no-show-raw-insn --print-imm-hex
```

``` text
{{#include ../ci/asm/app/release.objdump}}
```

> **NOTE:** To make this disassembly smaller I commented out the initialization
> of RAM.

Now look at the vector table. The 4th entry should be the address of
`HardFaultTrampoline` plus one.

``` console
$ cargo objdump --bin app --release -- -s -j .vector_table
```

``` text
{{#include ../ci/asm/app/release.vector_table}}
```

## `.o` / `.a` files

The downside of using the `cc` crate is that it requires some assembler program
on the build machine. For example when targeting ARM Cortex-M the `cc` crate
uses `arm-none-eabi-gcc` as the assembler.

Instead of assembling the file on the build machine we can ship a pre-assembled
file with the `rt` crate. That way no assembler program is required on the build
machine. However, you would still need an assembler on the machine that packages
and publishes the crate.

There's not much difference between an assembly (`.s`) file and its *compiled*
version: the object (`.o`) file. The assembler doesn't do any optimization; it
simply chooses the right object file format for the target architecture.

Cargo provides support for bundling archives (`.a`) with crates. We can package
object files into an archive using the `ar` command and then bundle the archive
with the crate. In fact, this what the `cc` crate does; you can see the commands
it invoked by searching for a file named `output` in the `target` directory.

``` console
$ grep running $(find target -name output)
```

``` text
running: "arm-none-eabi-gcc" "-O0" "-ffunction-sections" "-fdata-sections" "-fPIC" "-g" "-fno-omit-frame-pointer" "-mthumb" "-march=armv7-m" "-Wall" "-Wextra" "-o" "/tmp/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out/asm.o" "-c" "asm.s"
running: "ar" "crs" "/tmp/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out/libasm.a" "/home/japaric/rust-embedded/embedonomicon/ci/asm/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out/asm.o"
```

``` console
$ grep cargo $(find target -name output)
```

``` tetx
cargo:rustc-link-search=/tmp/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out
cargo:rustc-link-lib=static=asm
cargo:rustc-link-search=native=/tmp/app/target/thumbv7m-none-eabi/debug/build/rt-6ee84e54724f2044/out
```

We'll do something similar to produce an archive.

``` console
$ # most of flags `cc` uses have no effect when assembling so we drop them
$ arm-none-eabi-as -march=armv7-m asm.s -o asm.o

$ ar crs librt.a asm.o

$ arm-none-eabi-objdump -Cd librt.a
```

``` text
{{#include ../ci/asm/rt2/librt.objdump}}
```

Next we modify the build script to bundle this archive with the `rt` rlib.

``` console
$ cat ../rt/build.rs
```

``` rust
{{#include ../ci/asm/rt2/build.rs}}
```

Now we can test this new version against the simple program from before and
we'll get the same output.

``` console
$ cargo objdump --bin app --release -- -d --no-show-raw-insn --print-imm-hex
```

``` text
{{#include ../ci/asm/app2/release.objdump}}
```

> **NOTE**: As before I have commented out the RAM initialization to make the
> disassembly smaller.

``` console
$ cargo objdump --bin app --release -- -s -j .vector_table
```

``` text
{{#include ../ci/asm/app2/release.vector_table}}
```

The downside of shipping pre-assembled archives is that, in the worst case
scenario, you'll need to ship one build artifact for each compilation target
your library supports.
