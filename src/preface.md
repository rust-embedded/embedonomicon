# The embedonomicon

## Preface

The embedonomicon walks you through the process of creating a `#![no_std]` application from scratch
and through the iterative process of building architecture-specific functionality for Cortex-M
microcontrollers.

### Objectives

By reading this book you will learn

- How to build a `#![no_std]` application. This is much more complex than building a `#![no_std]`
  library because the target system may not be running an OS (or you could be aiming to build an
  OS!) and the program could be only process running in the target (or the first one), in which case
  the program may need to be customized for the target system.

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

This book is self contained. The reader doesn't need to be familiar with the Cortex-M architecture
nor access to a Cortex-M microcontroller -- all the examples included in this book can be tested in
QEMU. You will, however, need to install the following tools to run and inspect the examples in this
book:

- [`cargo-binutils`](https://github.com/japaric/cargo-binutils). Install it with `cargo install
  cargo-binutils`; then run `rustup component add llvm-tools-preview`.

- [`cargo-edit`](https://crates.io/crates/cargo-edit). Install it with `cargo install cargo-edit`.

- QEMU with support for ARM emulation. The `qemu-system-arm` program must be installed on your
  computer.

- LLDB. GDB with ARM support can also be used, but this book chooses LLDB as it's more likely that
  readers that are not into Cortex-M development have LLDB installed than it's likely that they have
  GDB with ARM support installed.
