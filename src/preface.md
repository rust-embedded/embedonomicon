# The embedonomicon

## Preface

The embedonomicon walks you through the process of creating a `#![no_std]` application from scratch
and through the iterative process of building architecture-specific functionality for Cortex-M
microcontrollers.

### Objectives

By reading this book you will learn

- How to build a `#![no_std]` application. This is much more complex than building a `#![no_std]`
  library because the target system may not be running an OS (or you could be aiming to build an
  OS!) and the program could be the only process running in the target (or the first one).
  In that case, the program may need to be customized for the target system.

- Tricks to finely control the memory layout of a Rust program. You'll learn about linkers, linker
  scripts and about the Rust features that let you control a bit of the ABI of Rust programs.

- A trick to implement default functionality that can be statically overridden (no runtime cost).

### Target audience

This book mainly targets to two audiences:

- People that wish to bootstrap bare metal support for an architecture that the ecosystem doesn't
  yet support (e.g. Cortex-R as of Rust 1.28), or for an architecture that Rust just gained support
  for (e.g. maybe Xtensa some time in the future).

- People that are curious about the unusual implementation of *runtime* crates like [`cortex-m-rt`],
  [`msp430-rt`] and [`riscv-rt`].

[`cortex-m-rt`]: https://crates.io/crates/cortex-m-rt
[`msp430-rt`]: https://crates.io/crates/msp430-rt
[`riscv-rt`]: https://crates.io/crates/riscv-rt

### Requirements

This book is self contained. The reader doesn't need to be familiar with the Cortex-M architecture,
nor is access to a Cortex-M microcontroller needed -- all the examples included in this book can be tested in
QEMU. You will, however, need to install the following tools to run and inspect the examples in this
book:

- All the code in this book uses the 2018 edition. If you are not familiar with
the 2018 features and idioms check [the edition guide]. Please also note that
until the 2018 edition is officially released you'll have to *manually modify
the Cargo.toml of new projects* to make use the 2018 edition. The required
changes are shown below:

[the edition guide]: https://rust-lang-nursery.github.io/edition-guide/

``` diff
+cargo-features = ["edition"]
+
 [package]
+edition = "2018"
 name = "hello"
 version = "0.1.0"
```

- A nightly toolchain from 2018-08-28 or newer.

- [`cargo-binutils`](https://github.com/japaric/cargo-binutils). v0.1.2 or newer.

- [`cargo-edit`](https://crates.io/crates/cargo-edit).

- The `thumbv7m-none-eabi` target.

- QEMU with support for ARM emulation. The `qemu-system-arm` program must be installed on your
  computer. The name may differ for non-Debian based distributions.

- LLDB. GDB with ARM support can also be used, but this book chooses LLDB as it's more likely that
  readers that are not into Cortex-M development have installed LLDB than GDB with ARM support.

  #### Rust toolchain setup on Linux

  ```bash
  rustup default nightly # If you start from scratch, get rustup from https://rustup.rs/
  rustup target add thumbv7m-none-eabi
  cargo install cargo-binutils
  rustup component add llvm-tools-preview
  sudo apt-get install libssl-dev # For Debian based systems (Ubuntu)
  cargo install cargo-edit
  ```
