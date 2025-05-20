# Logging with symbols

This section will show you how to utilize symbols and the ELF format to achieve
super cheap logging.

## Arbitrary symbols

Whenever we needed a stable symbol interface between crates we have mainly used
the `no_mangle` attribute and sometimes the `export_name` attribute. The
`export_name` attribute takes a string which becomes the name of the symbol
whereas `#[no_mangle]` is basically sugar for `#[export_name = <item-name>]`.

Turns out we are not limited to single word names; we can use arbitrary strings,
e.g. sentences, as the argument of the `export_name` attribute. As least when
the output format is ELF anything that doesn't contain a null byte is fine.

Let's check that out:

``` console
$ cargo new --lib foo

$ cat foo/src/lib.rs
```

``` rust
#[export_name = "Hello, world!"]
#[used]
static A: u8 = 0;

#[export_name = "こんにちは"]
#[used]
static B: u8 = 0;
```

``` console
$ ( cd foo && cargo nm --lib )
foo-d26a39c34b4e80ce.3lnzqy0jbpxj4pld.rcgu.o:
0000000000000000 r Hello, world!
0000000000000000 V __rustc_debug_gdb_scripts_section__
0000000000000000 r こんにちは
```

Can you see where this is going?

## Encoding

Here's what we'll do: we'll create one `static` variable per log message but
instead of storing the messages *in* the variables we'll store the messages in
the variables' *symbol names*. What we'll log then will not be the contents of
the `static` variables but their addresses.

As long as the `static` variables are not zero sized each one will have a
different address. What we're doing here is effectively encoding each message
into a unique identifier, which happens to be the variable address. Some part of
the log system will have to decode this id back into the message.

Let's write some code to illustrate the idea.

In this example we'll need some way to do I/O so we'll use the
[`cortex-m-semihosting`] crate for that. Semihosting is a technique for having a
target device borrow the host I/O capabilities; the host here usually refers to
the machine that's debugging the target device. In our case, QEMU supports
semihosting out of the box so there's no need for a debugger. On a real device
you'll have other ways to do I/O like a serial port; we use semihosting in this
case because it's the easiest way to do I/O on QEMU.

[`cortex-m-semihosting`]: https://crates.io/crates/cortex-m-semihosting

Here's the code

``` rust
{{#include ../ci/logging/app/src/main.rs}}
```

We also make use of the `debug::exit` API to have the program terminate the QEMU
process. This is a convenience so we don't have to manually terminate the QEMU
process.

And here's the `dependencies` section of the Cargo.toml:

``` toml
{{#include ../ci/logging/app/Cargo.toml:7:9}}
```

Now we can build the program

``` console
$ cargo build
```

To run it we'll have to add the `--semihosting-config` flag to our QEMU
invocation:

``` console
$ qemu-system-arm \
      -cpu cortex-m3 \
      -machine lm3s6965evb \
      -nographic \
      -semihosting-config enable=on,target=native \
      -kernel target/thumbv7m-none-eabi/debug/app
```

``` text
{{#include ../ci/logging/app/dev.out}}
```

> **NOTE**: These addresses may not be the ones you get locally because
> addresses of `static` variable are not guaranteed to remain the same when the
> toolchain is changed (e.g. optimizations may have improved).

Now we have two addresses printed to the console.

## Decoding

How do we convert these addresses into strings? The answer is in the symbol
table of the ELF file.

``` console
$ cargo objdump --bin app -- -t | grep '\.rodata\s*0*1\b'
```

``` text
{{#include ../ci/logging/app/dev.objdump}}
$ # first column is the symbol address; last column is the symbol name
```

`objdump -t` prints the symbol table. This table contains *all* the symbols but
we are only looking for the ones in the `.rodata` section and whose size is one
byte (our variables have type `u8`).

It's important to note that the address of the symbols will likely change when
optimizing the program. Let's check that.

> **PROTIP** You can set `target.thumbv7m-none-eabi.runner` to the long QEMU
> command from before (`qemu-system-arm -cpu (..) -kernel`) in the Cargo
> configuration file (`.cargo/config.toml`) to have `cargo run` use that *runner* to
> execute the output binary.

``` console
$ head -n2 .cargo/config.toml
```

``` toml
{{#include ../ci/logging/app/.cargo/config.toml:1:2}}
```

``` console
$ cargo run --release
     Running `qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel target/thumbv7m-none-eabi/release/app`
```

``` text
{{#include ../ci/logging/app/release.out}}
```

``` console
$ cargo objdump --bin app --release -- -t | grep '\.rodata\s*0*1\b'
```

``` text
{{#include ../ci/logging/app/release.objdump}}
```

So make sure to always look for the strings in the ELF file you executed.

Of course, the process of looking up the strings in the ELF file can be automated
using a tool that parses the symbol table (`.symtab` section) contained in the
ELF file. Implementing such tool is out of scope for this book and it's left as
an exercise for the reader.

## Making it zero cost

Can we do better? Yes, we can!

The current implementation places the `static` variables in `.rodata`, which
means they occupy size in Flash even though we never use their contents. Using a
little bit of linker script magic we can make them occupy *zero* space in Flash.

``` console
$ cat log.x
```

``` text
{{#include ../ci/logging/app2/log.x}}
```

We'll place the `static` variables in this new output `.log` section. This
linker script will collect all the symbols in the `.log` sections of input
object files and put them in an output `.log` section. We have seen this pattern
in the [Memory layout] chapter.

[Memory layout]: memory-layout.html

The new bit here is the `(INFO)` part; this tells the linker that this section
is a non-allocatable section. Non-allocatable sections are kept in the ELF
binary as metadata but they are not loaded onto the target device.

We also specified the start address of this output section: the `0` in `.log 0
(INFO)`.

The other improvement we can do is switch from formatted I/O (`fmt::Write`) to
binary I/O, that is send the addresses to the host as bytes rather than as
strings.

Binary serialization can be hard but we'll keep things super simple by
serializing each address as a single byte. With this approach we don't have to
worry about endianness or framing. The downside of this format is that a single
byte can only represent up to 256 different addresses.

Let's make those changes:

``` rust
{{#include ../ci/logging/app2/src/main.rs}}
```

Before you run this you'll have to append `-Tlog.x` to the arguments passed to
the linker. That can be done in the Cargo configuration file.

``` console
$ cat .cargo/config.toml
```

``` toml
{{#include ../ci/logging/app2/.cargo/config.toml}}
```

Now you can run it! Since the output now has a binary format we'll pipe it
through the `xxd` command to reformat it as a hexadecimal string.

``` console
$ cargo run | xxd -p
```

``` text
{{#include ../ci/logging/app2/dev.out}}
```

The addresses are `0x00` and `0x01`. Let's now look at the symbol table.

``` console
$ cargo objdump --bin app -- -t | grep '\.log'
```

``` text
{{#include ../ci/logging/app2/dev.objdump}}
```

There are our strings. You'll notice that their addresses now start at zero;
this is because we set a start address for the output `.log` section.

Each variable is 1 byte in size because we are using `u8` as their type. If we
used something like `u16` then all address would be even and we would not be
able to efficiently use all the address space (`0...255`).

## Packaging it up

You've noticed that the steps to log a string are always the same so we can
refactor them into a macro that lives in its own crate. Also, we can make the
logging library more reusable by abstracting the I/O part behind a trait.

``` console
$ cargo new --lib log

$ cat log/src/lib.rs
```

``` rust
{{#include ../ci/logging/log/src/lib.rs}}
```

Given that this library depends on the `.log` section it should be its
responsibility to provide the `log.x` linker script so let's make that happen.

``` console
$ mv log.x ../log/
```

``` console
$ cat ../log/build.rs
```

``` rust
{{#include ../ci/logging/log/build.rs}}
```

Now we can refactor our application to use the `log!` macro:

``` console
$ cat src/main.rs
```

``` rust
{{#include ../ci/logging/app3/src/main.rs}}
```

Don't forget to update the `Cargo.toml` file to depend on the new `log` crate.

``` console
$ tail -n4 Cargo.toml
```

``` toml
{{#include ../ci/logging/app3/Cargo.toml:7:10}}
```

``` console
$ cargo run | xxd -p
```

``` text
{{#include ../ci/logging/app3/dev.out}}
```

``` console
$ cargo objdump --bin app -- -t | grep '\.log'
```

``` text
{{#include ../ci/logging/app3/dev.objdump}}
```

Same output as before!

## Bonus: Multiple log levels

Many logging frameworks provide ways to log messages at different *log levels*.
These log levels convey the severity of the message: "this is an error", "this
is just a warning", etc. These log levels can be used to filter out unimportant
messages when searching for e.g. error messages.

We can extend our logging library to support log levels without increasing its
footprint. Here's how we'll do that:

We have a flat address space for the messages: from `0` to `255` (inclusive). To
keep things simple let's say we only want to differentiate between error
messages and warning messages. We can place all the error messages at the
beginning of the address space, and all the warning messages *after* the error
messages. If the decoder knows the address of the first warning message then it
can classify the messages. This idea can be extended to support more than two
log levels.

Let's test the idea by replacing the `log` macro with two new macros: `error!`
and `warn!`.

``` console
$ cat ../log/src/lib.rs
```

``` rust
{{#include ../ci/logging/log2/src/lib.rs}}
```

We distinguish errors from warnings by placing the messages in different link
sections.

The next thing we have to do is update the linker script to place error messages
before the warning messages.

``` console
$ cat ../log/log.x
```

``` text
{{#include ../ci/logging/log2/log.x}}
```

We also give a name, `__log_warning_start__`, to the boundary between the errors
and the warnings. The address of this symbol will be the address of the first
warning message.

We can now update the application to make use of these new macros.

``` console
$ cat src/main.rs
```

``` rust
{{#include ../ci/logging/app4/src/main.rs}}
```

The output won't change much:

``` console
$ cargo run | xxd -p
```

``` text
{{#include ../ci/logging/app4/dev.out}}
```

We still get two bytes in the output but the error is given the address 0 and
the warning is given the address 1 even though the warning was logged first.

Now look at the symbol table.

```  console
$ cargo objdump --bin app -- -t | grep '\.log'
```

``` text
{{#include ../ci/logging/app4/dev.objdump}}
```

There's now an extra symbol, `__log_warning_start__`, in the `.log` section.
The address of this symbol is the address of the first warning message.
Symbols with addresses lower than this value are errors, and the rest of symbols
are warnings.

With an appropriate decoder you could get the following human readable output
from all this information:

``` text
WARNING Hello, world!
ERROR Goodbye
```

---

If you liked this section check out the [`stlog`] logging framework which is a
complete implementation of this idea.

[`stlog`]: https://crates.io/crates/stlog
