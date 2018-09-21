# The embedonomicon

The embedonomicon walks you through the process of creating a `#![no_std]` application from scratch
and through the iterative process of building architecture-specific functionality for Cortex-M
microcontrollers.

## Objectives

By reading this book you will learn

- How to build a `#![no_std]` application. This is much more complex than building a `#![no_std]`
  library because the target system may not be running an OS (or you could be aiming to build an
  OS!) and the program could be the only process running in the target (or the first one).
  In that case, the program may need to be customized for the target system.

- Tricks to finely control the memory layout of a Rust program. You'll learn about linkers, linker
  scripts and about the Rust features that let you control a bit of the ABI of Rust programs.

- A trick to implement default functionality that can be statically overridden (no runtime cost).

## Target audience

This book mainly targets to two audiences:

- People that wish to bootstrap bare metal support for an architecture that the ecosystem doesn't
  yet support (e.g. Cortex-R as of Rust 1.28), or for an architecture that Rust just gained support
  for (e.g. maybe Xtensa some time in the future).

- People that are curious about the unusual implementation of *runtime* crates like [`cortex-m-rt`],
  [`msp430-rt`] and [`riscv-rt`].

[`cortex-m-rt`]: https://crates.io/crates/cortex-m-rt
[`msp430-rt`]: https://crates.io/crates/msp430-rt
[`riscv-rt`]: https://crates.io/crates/riscv-rt

## Requirements

This book is self contained. The reader doesn't need to be familiar with the
Cortex-M architecture, nor is access to a Cortex-M microcontroller needed -- all
the examples included in this book can be tested in QEMU. You will, however,
need to install the following tools to run and inspect the examples in this
book:

- All the code in this book uses the 2018 edition. If you are not familiar with
  the 2018 features and idioms check [the edition guide].

- Rust 1.30, 1.30-beta, nightly-2018-09-13, or a newer toolchain PLUS ARM
  Cortex-M compilation support.

- [`cargo-binutils`](https://github.com/japaric/cargo-binutils). v0.1.4 or newer.

- [`cargo-edit`](https://crates.io/crates/cargo-edit).

- QEMU with support for ARM emulation. The `qemu-system-arm` program must be
  installed on your computer.

- GDB with ARM support.

### Example setup

Instructions common to all OSes

``` console
$ # Rust toolchain
$ # If you start from scratch, get rustup from https://rustup.rs/
$ rustup default beta

$ # toolchain should be newer than this one
$ rustc -V
rustc 1.30.0-beta.1 (14f51b05d 2018-09-18)

$ rustup target add thumbv7m-none-eabi

$ # cargo-binutils
$ cargo install cargo-binutils

$ rustup component add llvm-tools-preview

```

#### macOS

``` console
$ # arm-none-eabi-gdb
$ # you may need to run `brew tap Caskroom/tap` first
$ brew cask install gcc-arm-embedded

$ # QEMU
$ brew install qemu
```

#### Ubuntu 16.04

``` console
$ # arm-none-eabi-gdb
$ sudo apt-get install gdb-arm-none-eabi

$ # QEMU
$ sudo apt-get install qemu-system-arm
```

#### Windows

- [arm-none-eabi-gdb](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads).
  The GNU Arm Embedded Toolchain includes GDB.

- [QEMU](https://www.qemu.org/download/#windows)
