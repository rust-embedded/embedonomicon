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

$ head -n4 Cargo.toml
```

``` toml
{{#include ../ci/main/rt/Cargo.toml:1:4}}
```

The first change is to have the reset handler call an external `main` function:

``` console
$ head -n13 src/lib.rs
```

``` rust
{{#include ../ci/main/rt/src/lib.rs:1:13}}
```

We also drop the `#![no_main]` attribute has it has no effect on library crates.

> There's an orthogonal question that arises at this stage: Should the `rt`
> library provide a standard panicking behavior, or should it *not* provide a
> `#[panic_handler]` function and leave the end user choose the panicking
> behavior? This document won't delve into that question and for simplicity will
> leave the dummy `#[panic_handler]` function in the `rt` crate. However, we
> wanted to inform the reader that there are other options.

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
{{#include ../ci/main/rt/build.rs}}
```

Now the user can write an application that exposes the `main` symbol and link it to the `rt` crate.
The `rt` will take care of giving the program the right memory layout.

``` console
$ cd ..

$ cargo new --edition 2018 --bin app

$ cd app

$ # modify Cargo.toml to include the `rt` crate as a dependency
$ tail -n2 Cargo.toml
```

``` toml
{{#include ../ci/main/app/Cargo.toml:7:8}}
```

``` console
$ # copy over the config file that sets a default target and tweaks the linker invocation
$ cp -r ../rt/.cargo .

$ # change the contents of `main.rs` to
$ cat src/main.rs
```

``` rust
{{#include ../ci/main/app/src/main.rs}}
```

The disassembly will be similar but will now include the user `main` function.

``` console
$ cargo objdump --bin app -- -d -no-show-raw-insn
```

``` text
{{#include ../ci/main/app/app.objdump}}
```

## Making it type safe

The `main` interface works, but it's easy to get it wrong: For example, the user could write `main`
as a non-divergent function, and they would get no compile time error and undefined behavior (the
compiler will misoptimize the program).

We can add type safety by exposing a macro to the user instead of the symbol interface. In the
`rt` crate, we can write this macro:

``` console
$ tail -n12 ../rt/src/lib.rs
```

``` rust
{{#include ../ci/main/rt/src/lib.rs:25:37}}
```

Then the application writers can invoke it like this:

``` console
$ cat src/main.rs
```

``` rust
{{#include ../ci/main/app2/src/main.rs}}
```

Now the author will get an error if they change the signature of `main` to be
non divergent function, e.g. `fn()`.

## Life before main

`rt` is looking good but it's not feature complete! Applications written against it can't use
`static` variables or string literals because `rt`'s linker script doesn't define the standard
`.bss`, `.data` and `.rodata` sections. Let's fix that!

The first step is to define these sections in the linker script:

``` console
$ # showing just a fragment of the file
$ sed -n 25,46p ../rt/link.x
```

``` text
{{#include ../ci/main/rt/link.x:25:46}}
```

They just re-export the input sections and specify in which memory region each output section will
go.

With these changes, the following program will compile:

``` rust
{{#include ../ci/main/app3/src/main.rs}}
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

``` console
$ # showing just a fragment of the file
$ sed -n 25,52p ../rt/link.x
```

``` text
{{#include ../ci/main/rt2/link.x:25:52}}
```

Let's go into the details of these changes:

``` text
{{#include ../ci/main/rt2/link.x:38}}
```

``` text
{{#include ../ci/main/rt2/link.x:40}}
```

``` text
{{#include ../ci/main/rt2/link.x:45}}
```

``` text
{{#include ../ci/main/rt2/link.x:47}}
```

We associate symbols to the start and end addresses of the `.bss` and `.data` sections, which we'll
later use from Rust code.

``` text
{{#include ../ci/main/rt2/link.x:43}}
```

We set the Load Memory Address (LMA) of the `.data` section to the end of the `.rodata`
section. The `.data` contains `static` variables with a non-zero initial value; the Virtual Memory
Address (VMA) of the `.data` section is somewhere in RAM -- this is where the `static` variables are
located. The initial values of those `static` variables, however, must be allocated in non volatile
memory (Flash); the LMA is where in Flash those initial values are stored.

``` text
{{#include ../ci/main/rt2/link.x:50}}
```

Finally, we associate a symbol to the LMA of `.data`.

On the Rust side, we zero the `.bss` section and initialize the `.data` section. We can reference
the symbols we created in the linker script from the Rust code. The *addresses*[^1] of these symbols are
the boundaries of the `.bss` and `.data` sections.

The updated reset handler is shown below:

``` console
$ head -n32 ../rt/src/lib.rs
```

``` rust
{{#include ../ci/main/rt2/src/lib.rs:1:31}}
```

Now end users can directly and indirectly make use of `static` variables without running into
undefined behavior!

> In the code above we performed the memory initialization in a bytewise fashion. It's possible to
> force the `.bss` and `.data` sections to be aligned to, say, 4 bytes. This fact can then be used
> in the Rust code to perform the initialization wordwise while omitting alignment checks. If you
> are interested in learning how this can be achieved check the [`cortex-m-rt`] crate.

[`cortex-m-rt`]: https://github.com/japaric/cortex-m-rt/tree/v0.5.1

[^1]: The fact that the addresses of the linker script symbols must be used here can be confusing and
unintuitive. An elaborate explanation for this oddity can be found [here](https://stackoverflow.com/a/40392131).
