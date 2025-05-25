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

## Translations

This book has been translated by generous volunteers. If you would like your
translation listed here, please open a PR to add it.

* [Japanese](https://tomoyuki-nakabayashi.github.io/embedonomicon/)
  ([repository](https://github.com/tomoyuki-nakabayashi/embedonomicon))

* [Chinese](https://xxchang.github.io/embedonomicon/)
  ([repository](https://github.com/xxchang/embedonomicon))
  
## Requirements

This book is self contained. The reader doesn't need to be familiar with the
Cortex-M architecture, nor is access to a Cortex-M microcontroller needed -- all
the examples included in this book can be tested in QEMU. You will, however,
need to install the following tools to run and inspect the examples in this
book:

- All the code in this book uses the 2018 edition. If you are not familiar with
  the 2018 features and idioms check the [`edition guide`].

- Rust 1.31 or a newer toolchain PLUS ARM Cortex-M compilation support.

- [`cargo-binutils`](https://github.com/japaric/cargo-binutils). v0.1.4 or newer.

- [`cargo-edit`](https://crates.io/crates/cargo-edit).

- QEMU with support for ARM emulation. The `qemu-system-arm` program must be
  installed on your computer.

- GDB with ARM support.

[`edition guide`]: https://rust-lang-nursery.github.io/edition-guide/

### Example setup

Instructions common to all OSes

```
console
# Rust toolchain
# If you start from scratch, get rustup from https://rustup.rs/

# Change rustc to default stable version.
$ rustup default stable
$ rustc -V
$ rustc +nightly -V

# Change rustc to default nightly version.
$ rustup default nightly
$ rustc -V

# toolchain should be newer than this one.
$ rustc -V
rustc 1.59.0 (9d1b2106e 2022-02-23)

$ rustc +nightly -V
rustc 1.61.0-nightly (9c06e1ba4 2022-03-29)

$ rustup target add thumbv7m-none-eabi
$ rustup +nightly target add thumbv7m-none-eabi

# cargo-binutils
$ cargo install cargo-binutils
$ cargo +nightly install cargo-binutils

$ rustup component add llvm-tools-preview
$ rustup +nightly component add llvm-tools-preview

```

#### macOS

``` console
$ # arm-none-eabi-gdb
$ # you may need to run `brew tap Caskroom/tap` first
$ brew install --cask gcc-arm-embedded

$ # QEMU
$ brew install qemu
```

#### Ubuntu 16.04

``` console
$ # arm-none-eabi-gdb
$ sudo apt install gdb-arm-none-eabi

$ # QEMU
$ sudo apt install qemu-system-arm
```

#### Ubuntu 18.04 or Debian

``` console
$ # gdb-multiarch -- use `gdb-multiarch` when you wish to invoke gdb
$ sudo apt install gdb-multiarch

$ # QEMU
$ sudo apt install qemu-system-arm
```

#### Windows

- [arm-none-eabi-gdb](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads).
  The GNU Arm Embedded Toolchain includes GDB.

- [QEMU](https://www.qemu.org/download/#windows)

## Installing a toolchain bundle from ARM (optional step) (tested on Ubuntu 18.04)
- With the late 2018 switch from
[GCC's linker to LLD](https://rust-embedded.github.io/blog/2018-08-2x-psa-cortex-m-breakage/) for Cortex-M 
microcontrollers, [gcc-arm-none-eabi][1] is no longer 
required.  But for those wishing to use the toolchain 
anyway, install from [here][1] and follow the steps outlined below:
``` console
$ tar xvjf gcc-arm-none-eabi-8-2018-q4-major-linux.tar.bz2
$ mv gcc-arm-none-eabi-<version_downloaded> <your_desired_path> # optional
$ export PATH=${PATH}:<path_to_arm_none_eabi_folder>/bin # add this line to .bashrc to make persistent
```
[1]: https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads
