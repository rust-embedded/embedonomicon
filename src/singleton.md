# Global singletons

In this section we'll cover how to implement a global, shared singleton. The
embedded Rust book covered local, owned singletons which are pretty much unique
to Rust. Global singletons are essentially the singleton pattern you see in C
and C++; they are not specific to embedded development but since they involve
symbols they seemed a good fit for the Embedonomicon.

> **TODO**(resources team) link "the embedded Rust book" to the singletons
> section when it's up

To illustrate this section we'll extend the logger we developed in the last
section to support global logging. The result will be very similar to the
`#[global_allocator]` feature covered in the embedded Rust book.

> **TODO**(resources team) link `#[global_allocator]` to the collections chapter
> of the book when it's in a more stable location.

Here's the summary of what we want to do:

In the last section we created a `log!` macro to log messages through a specific
logger, a value that implements the `Log` trait. The syntax of the `log!` macro
is `log!(logger, "String")`. We want to extend the macro such that
`log!("String")` also works. Using the `logger`-less version should log the
message through a global logger; this is how `std::println!` works. We'll also
need a mechanism to declare what the global logger is; this is the part that's
similar to `#[global_allocator]`.

It could be that the global logger is declared in the top crate, and it could
also be that the type of the global logger is defined in the top crate. In this
scenario the dependencies *cannot* know the exact type of the global logger. To
support this scenario we'll need some indirection.

Instead of hardcoding the type of the global logger in the `log` crate we'll
declare only the *interface* of the global logger in that crate. That is we'll
add a new trait, `GlobalLog`, to the `log` crate. The `log!` macro will also
have to make use of that trait.

``` console
$ cat ../log/src/lib.rs
```

``` rust
{{#include ../ci/singleton/log/src/lib.rs}}
```

There's quite a bit to unpack here.

Let's start with the trait.

``` rust
{{#include ../ci/singleton/log/src/lib.rs:4:6}}
```

Both `GlobalLog` and `Log` have a `log` method. The difference is that
`GlobalLog.log` takes a shared reference to the receiver (`&self`). This is
necessary because the global logger will be a `static` variable. More on that
later.

The other difference is that `GlobalLog.log` doesn't return a `Result`. This
means that it can *not* report errors to the caller. This is not a strict
requirement for traits used to implement global singletons. Error handling in
global singletons is fine but then all users of the global version of the `log!`
macro have to agree on the error type. Here we are simplifying the interface a
bit by having the `GlobalLog` implementer deal with the errors.

Yet another difference is that `GlobalLog` requires that the implementer is
`Sync`, that is that it can be shared between threads. This is a requirement for
values placed in `static` variables; their types must implement the `Sync`
trait.

At this point it may not be entirely clear why the interface has to look this
way. The other parts of the crate will make this clearer so keep reading.

Next up is the `log!` macro:

``` rust
{{#include ../ci/singleton/log/src/lib.rs:17:29}}
```

When called without a specific `$logger` the macros uses an `extern` `static`
variable called `LOGGER` to log the message. This variable *is* the global
logger that's defined somewhere else; that's why we use the `extern` block. We
saw this pattern in the [main interface] chapter.

[main interface]: main.html

We need to declare a type for `LOGGER` or the code won't type check. We don't
know the concrete type of `LOGGER` at this point but we know, or rather require,
that it implements the `GlobalLog` trait so we can use a trait object here.

The rest of the macro expansion looks very similar to the expansion of the local
version of the `log!` macro so I won't explain it here as it's explained in the
[previous] chapter.

[previous]: logging.html

Now that we know that `LOGGER` has to be a trait object it's clearer why we
omitted the associated `Error` type in `GlobalLog`. If we had not omitted then
we would have need to pick a type for `Error` in the type signature of `LOGGER`.
This is what I earlier meant by "all users of `log!` would need to agree on the
error type".

Now the final piece: the `global_logger!` macro. It could have been a proc macro
attribute but it's easier to write a `macro_rules!` macro.

``` rust
{{#include ../ci/singleton/log/src/lib.rs:41:47}}
```

This macro creates the `LOGGER` variable that `log!` uses. Because we need a
stable ABI interface we use the `no_mangle` attribute. This way the symbol name
of `LOGGER` will be "LOGGER" which is what the `log!` macro expects.

The other important bit is that the type of this static variable must exactly
match the type used in the expansion of the `log!` macro. If they don't match
Bad Stuff will happen due to ABI mismatch.

Let's write an example that uses this new global logger functionality.

``` console
$ cat src/main.rs
```

``` rust
{{#include ../ci/singleton/app/src/main.rs}}
```

> **TODO**(resources team) use `cortex_m::Mutex` instead of a `static mut`
> variable when `const fn` is stabilized.

We had to add `cortex-m` to the dependencies.

``` console
$ tail -n5 Cargo.toml
```

``` text
{{#include ../ci/singleton/app/Cargo.toml:11:15}}
```

This is a port of one of the examples written in the [previous] section. The
output is the same as what we got back there.

``` console
$ cargo run | xxd -p
```

``` text
{{#include ../ci/singleton/app/dev.out}}
```

``` console
$ cargo objdump --bin app -- -t | grep '\.log'
```

``` text
{{#include ../ci/singleton/app/dev.objdump}}
```

---

Some readers may be concerned about this implementation of global singletons not
being zero cost because it uses trait objects which involve dynamic dispatch,
that is method calls are performed through a vtable lookup.

However, it appears that LLVM is smart enough to eliminate the dynamic dispatch
when compiling with optimizations / LTO. This can be confirmed by searching for
`LOGGER` in the symbol table.

``` console
$ cargo objdump --bin app --release -- -t | grep LOGGER
```

``` text
{{#include ../ci/singleton/app/release.objdump}}
```

If the `static` is missing that means that there is no vtable and that LLVM was
capable of transforming all the `LOGGER.log` calls into `Logger.log` calls.
