# A `main` interface

We have a minimal working program now, but we need to package it in a way that the end user can build
safe programs on top of it. In this section, we'll implement a `main` interface like the one standard
Rust programs use.

First, we'll convert our binary crate into a library crate:

``` console
$ mv src/main.rs src/lib.rs
```

And then rename it to `rt` which stands for "runtime".

``` console
$ sed -i s/app/rt/ Cargo.toml

$ head -n2 Cargo.toml
[package]
name = "rt"
```

The first change is to have the reset handler call an external `main` function:

``` rust
// #![no_main]

#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    extern "Rust" {
        fn main() -> !;
    }

    main()
}
```

We also drop the `#![no_main]` attribute has it has no effect on library crates.

> There's an orthogonal question that arises at this stage: Should the `rt` library provide a
> standard panicking behavior, or should it *not* provide a `#[panic_implementation]` function and
> leave the end user choose the panicking behavior? This document won't delve into that question and
> for simplicity will leave the dummy `#[panic_implementation]` function in the `rt` crate.
> However, we wanted to inform the reader that there are other options.

The second change involves providing the linker script we wrote before to the application crate. You
see the linker will search for linker scripts in the library search path (`-L`) and in the directory
from which it's invoked. The application crate shouldn't need to carry around a copy of `link.x` so
we'll have the `rt` crate put the linker script in the library search path using a [build script].

[build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html

``` console
$ # create a build.rs file in the root of `rt` with these contents
$ cat build.rs
```

``` rust
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<Error>> {
    // build directory for this crate
    let mut out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // extend the library search path
    println!("cargo:rustc-link-search={}", out.display());

    // put `link.x` in the build directory
    out.push("link.x");
    File::create(out)?.write_all(include_bytes!("link.x"))?;

    Ok(())
}
```

Now the user can write an application that exposes the `main` symbol and link it to the `rt` crate.
The `rt` will take care of giving the program the right memory layout.

``` console
$ cargo new --bin app

$ cd app

$ cargo add rt --path ../rt

$ # copy over the Cargo config file that sets the default build target
$ cp -r ../rt/.cargo .

$ # change the contents of `main.rs` to
$ cat src/main.rs
```

``` rust
#![no_std]
#![no_main]

extern crate rt;

#[no_mangle]
pub fn main() -> ! {
    let x = 42;

    loop {}
}
```

``` console
$ cargo rustc -- \
      -C linker=rust-lld \
      -Z linker-flavor=ld.lld \
      -C link-arg=-Tlink.x

$ cargo objdump -- \
      -d -no-show-raw-insn target/thumbv7m-none-eabi/debug/app

target/thumbv7m-none-eabi/debug/app:    file format ELF32-arm-little

Disassembly of section .text:
main:
       8:       sub     sp, #4
       a:       movs    r0, #42
       c:       str     r0, [sp]
       e:       b       #-2 <main+0x8>
      10:       b       #-4 <main+0x8>

Reset:
      12:       bl      #-14
      16:       trap
```

## Making it type safe

The `main` interface works, but it's easy to get it wrong: For example, the user could write `main`
as a non-divergent function, and they would get no compile time error and undefined behavior (the
compiler will misoptimize the program).

We can add type safety by exposing a macro to the user instead of the symbol interface. In the
`rt` crate, we can write this macro:

``` rust
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[export_name = "main"]
        pub unsafe fn __main() -> ! {
            // type check the given path
            let f: fn() -> ! = $path;

            f()
        }
    }
}
```

Then the application writers can invoke it like this:

``` rust
#![no_std]
#![no_main]

#[macro_use]
extern crate rt;

entry!(main);

fn main() -> ! {
    let x = 42;

    loop {}
}
```

Now the author will get an error if they change the signature of `main` to be non divergent, e.g.
`fn()`.

## Life before main

`rt` is looking good but it's not feature complete! Applications written against it can't use
`static` variables or string literals because `rt`'s linker script doesn't define the standard
`.bss`, `.data` and `.rodata` sections. Let's fix that!

The first step is to define these sections in the linker script:

```
  /* inside SECTIONS; after .text */

  .rodata :
  {
    *(.rodata .rodata.*);
  } > FLASH

  .bss :
  {
    *(.bss .bss.*);
  } > RAM

  .data :
  {
    *(.data .data.*);
  } > RAM
```

They just re-export the input sections and specify in which memory region each output section will
go.

With these changes, the following program will compile:

``` rust
#![no_std]
#![no_main]

#[macro_use]
extern crate rt;

entry!(main);

static RODATA: &[u8] = b"Hello, world!";
static mut BSS: u8 = 0;
static mut DATA: u16 = 1;

fn main() -> ! {
    let _x = RODATA;
    let _y = unsafe { &BSS };
    let _z = unsafe { &DATA };

    loop {}
}
```

However if you run this program on real hardware and debug it, you'll observe that the `static`
variables `BSS` and `DATA` don't have the values `0` and `1` by the time `main` has been reached.
Instead, these variables will have junk values. The problem is that the contents of RAM are
random after powering up the device. You won't be able to observe this effect if you run the
program in QEMU.

As things stand if your program reads any `static` variable before performing a write to it then
your program has undefined behavior. Let's fix that by initializing all `static` variables before
calling `main`.

We'll need to tweak the linker script a bit more to do the RAM initialization:

```
  .rodata :
  {
    *(.rodata .rodata.*);
  } > FLASH

  .bss :
  {
    _sbss = .;
    *(.bss .bss.*);
    _ebss = .;
  } > RAM

  .data : AT(ADDR(.rodata) + SIZEOF(.rodata))
  {
    _sdata = .;
    *(.data .data.*);
    _edata = .;
  } > RAM

  _sidata = LOADADDR(.data);
```

Let's go into the details of these changes:

```
    _sbss = .;
    _ebss = .;
    _sdata = .;
    _edata = .;
```

We associate symbols to the start and end addresses of the `.bss` and `.data` sections, which we'll
later use from Rust code.

```
  .data : AT(ADDR(.rodata) + SIZEOF(.rodata))
```

We set the Load Memory Address (LMA) of the `.data` section to the end of the `.rodata`
section. The `.data` contains `static` variables with a non-zero initial value; the Virtual Memory
Address (VMA) of the `.data` section is somewhere in RAM -- this is where the `static` variables are
located. The initial values of those `static` variables, however, must be allocated in non volatile
memory (Flash); the LMA is where in Flash those initial values are stored.

```
  _sidata = LOADADDR(.data);
```

Finally, we associate a symbol to the LMA of `.data`.

On the Rust side, we zero the `.bss` section and initialize the `.data` section. We can reference
the symbols we created in the linker script from the Rust code. The *addresses*[^1] of these symbols are
the boundaries of the `.bss` and `.data` sections.

[^1]: The fact that the addresses of the linker script symbols must be used here can be confusing and
unintuitive. An elaborate explanation for this oddity can be found [here](https://stackoverflow.com/a/40392131).

The updated reset handler is shown below:

``` rust
use core::ptr;

#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    // Initialize RAM
    extern "C" {
        static mut _sbss: u8;
        static mut _ebss: u8;

        static mut _sdata: u8;
        static mut _edata: u8;
        static _sidata: u8;
    }

    let count = &_ebss as *const u8 as usize - &_sbss as *const u8 as usize;
    ptr::write_bytes(&mut _sbss as *mut u8, 0, count);

    let count = &_edata as *const u8 as usize - &_sdata as *const u8 as usize;
    ptr::copy_nonoverlapping(&_sidata as *const u8, &mut _sdata as *mut u8, count);

    // Call user entry point
    extern "Rust" {
        fn main() -> !;
    }

    main()
}
```

Now end users can directly and indirectly make use of `static` variables without running into
undefined behavior!

> In the code above we performed the memory initialization in a bytewise fashion. It's possible to
> force the `.bss` and `.data` sections to be aligned to, say, 4 bytes. This fact can then be used
> in the Rust code to perform the initialization wordwise while omitting alignment checks. If you
> are interested in learning how this can be achieved check the [`cortex-m-rt`] crate.

[`cortex-m-rt`]: https://github.com/japaric/cortex-m-rt/tree/v0.5.1
