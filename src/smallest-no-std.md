# The smallest `#![no_std]` program

In this section we'll write the smallest `#![no_std]` program that *compiles*.

## What does `#![no_std]` mean?

`#![no_std]` is a crate level attribute that indicates that the crate will link to the [`core`] crate
instead of the [`std`] crate, but what does this mean for applications?

[`core`]: https://doc.rust-lang.org/core/
[`std`]: https://doc.rust-lang.org/std/

The `std` crate is Rust's standard library. It contains functionality that assumes that the program
will run on an operating system rather than [*directly on the metal*]. `std` also assumes that the
operating system is a general purpose operating system, like the ones one would find in servers and
desktops. For this reason, `std` provides a standard API over functionality one usually finds in
such operating systems: Threads, files, sockets, a filesystem, processes, etc.

[*directly on the metal*]: https://en.wikipedia.org/wiki/Bare_machine

On the other hand, the `core` crate is a subset of the `std` crate that makes zero assumptions about
the system the program will run on. As such, it provides APIs for language primitives like floats,
strings and slices, as well as APIs that expose processor features like atomic operations and SIMD
instructions. However it lacks APIs for anything that involves heap memory allocations and I/O.

For an application, `std` does more than just providing a way to access OS abstractions. `std` also
takes care of, among other things, setting up stack overflow protection, processing command line
arguments and spawning the main thread before a program's `main` function is invoked. A `#![no_std]`
application lacks all that standard runtime, so it must initialize its own runtime, if any is
required.

Because of these properties, a `#![no_std]` application can be the first and / or the only code that
runs on a system. It can be many things that a standard Rust application can never be, for example:
- The kernel of an OS.
- Firmware.
- A bootloader.

## The code

With that out of the way, we can move on to the smallest `#![no_std]` program that compiles:

``` console
$ cargo new --edition 2018 --bin app

$ cd app
```

``` console
$ # modify main.rs so it has these contents
$ cat src/main.rs
```

``` rust
{{#include ../ci/smallest-no-std/src/main.rs}}
```

This program contains some things that you won't see in standard Rust programs:

The `#![no_std]` attribute which we have already extensively covered.

The `#![no_main]` attribute which means that the program won't use the standard `main` function as
its entry point. At the time of writing, Rust's `main` interface makes some assumptions about the
environment the program executes in: For example, it assumes the existence of command line
arguments, so in general, it's not appropriate for `#![no_std]` programs.

The `#[panic_handler]` attribute. The function marked with this attribute defines the behavior
of panics, both library level panics (`core::panic!`) and language level panics (out of bounds
indexing).

This program doesn't produce anything useful. In fact, it will produce an empty binary.

``` console
$ # equivalent to `size target/thumbv7m-none-eabi/debug/app`
$ cargo size --target thumbv7m-none-eabi --bin app
```

``` text
{{#include ../ci/smallest-no-std/app.size}}
```

Before linking the crate does contain the panicking symbol.

``` console
$ cargo rustc --target thumbv7m-none-eabi -- --emit=obj

$ cargo nm -- target/thumbv7m-none-eabi/debug/deps/app-*.o | grep '[0-9]* [^n] '
```

``` text
{{#include ../ci/smallest-no-std/app.o.nm}}
```

However, it's our starting point. In the next section, we'll build something useful. But before
continuing, let's set a default build target to avoid having to pass the `--target` flag to every
Cargo invocation.

``` console
$ mkdir .cargo

$ # modify .cargo/config so it has these contents
$ cat .cargo/config
```

``` toml
{{#include ../ci/smallest-no-std/.cargo/config}}
```
