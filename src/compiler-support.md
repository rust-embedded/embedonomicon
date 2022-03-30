# A note on compiler support

This book makes use of a built-in *compiler* target, the `thumbv7m-none-eabi`, for which the Rust
team distributes a `rust-std` component, which is a pre-compiled collection of crates like [`core`]
and [`std`].

[`core`]: https://doc.rust-lang.org/core/index.html
[`std`]: https://doc.rust-lang.org/std/index.html

If you want to attempt replicating the contents of this book for a different target architecture,
you need to take into account the different levels of support that Rust provides for (compilation)
targets.

## LLVM support

As of Rust 1.28, the official Rust compiler, `rustc`, uses LLVM for (machine) code generation. The
minimal level of support Rust provides for an architecture is having its LLVM backend enabled in
`rustc`. You can see all the architectures that `rustc` supports, through LLVM, by running the
following command:

``` console
$ # you need to have `cargo-binutils` installed to run this command
$ cargo objdump -- --version
LLVM (http://llvm.org/):
  LLVM version 7.0.0svn
  Optimized build.
  Default target: x86_64-unknown-linux-gnu
  Host CPU: skylake

  Registered Targets:
    aarch64    - AArch64 (little endian)
    aarch64_be - AArch64 (big endian)
    arm        - ARM
    arm64      - ARM64 (little endian)
    armeb      - ARM (big endian)
    hexagon    - Hexagon
    mips       - Mips
    mips64     - Mips64 [experimental]
    mips64el   - Mips64el [experimental]
    mipsel     - Mipsel
    msp430     - MSP430 [experimental]
    nvptx      - NVIDIA PTX 32-bit
    nvptx64    - NVIDIA PTX 64-bit
    ppc32      - PowerPC 32
    ppc64      - PowerPC 64
    ppc64le    - PowerPC 64 LE
    sparc      - Sparc
    sparcel    - Sparc LE
    sparcv9    - Sparc V9
    systemz    - SystemZ
    thumb      - Thumb
    thumbeb    - Thumb (big endian)
    wasm32     - WebAssembly 32-bit
    wasm64     - WebAssembly 64-bit
    x86        - 32-bit X86: Pentium-Pro and above
    x86-64     - 64-bit X86: EM64T and AMD64
```

If LLVM supports the architecture you are interested in, but `rustc` is built with the backend
disabled (which is the case of AVR as of Rust 1.28), then you will need to modify the Rust source
enabling it. The first two commits of PR [rust-lang/rust#52787] give you an idea of the required
changes.

[rust-lang/rust#52787]: https://github.com/rust-lang/rust/pull/52787

On the other hand, if LLVM doesn't support the architecture, but a fork of LLVM does, you will have
to replace the original version of LLVM with the fork before building `rustc`. The Rust build system
allows this and in principle it should just require changing the `llvm` submodule to point to the
fork.

If your target architecture is only supported by some vendor provided GCC, you have the option of
using [`mrustc`], an unofficial Rust compiler, to translate your Rust program into C code and then
compile that using GCC.

[`mrustc`]: https://github.com/thepowersgang/mrustc

## Built-in target

A compilation target is more than just its architecture. Each target has a [specification]
associated to it that describes, among other things, its architecture, its operating system and the
default linker.

[specification]: https://github.com/rust-lang/rfcs/blob/master/text/0131-target-specification.md

The Rust compiler knows about several targets. These are *built into* the compiler and can be listed
by running the following command:

``` console
$ rustc --print target-list | column
aarch64-fuchsia                   mipsisa32r6el-unknown-linux-gnu
aarch64-linux-android             mipsisa64r6-unknown-linux-gnuabi64
aarch64-pc-windows-msvc           mipsisa64r6el-unknown-linux-gnuabi64
aarch64-unknown-cloudabi          msp430-none-elf
aarch64-unknown-freebsd           nvptx64-nvidia-cuda
aarch64-unknown-hermit            powerpc-unknown-linux-gnu
aarch64-unknown-linux-gnu         powerpc-unknown-linux-gnuspe
aarch64-unknown-linux-musl        powerpc-unknown-linux-musl
aarch64-unknown-netbsd            powerpc-unknown-netbsd
aarch64-unknown-none              powerpc-wrs-vxworks
aarch64-unknown-none-softfloat    powerpc-wrs-vxworks-spe
aarch64-unknown-openbsd           powerpc64-unknown-freebsd
aarch64-unknown-redox             powerpc64-unknown-linux-gnu
aarch64-uwp-windows-msvc          powerpc64-unknown-linux-musl
aarch64-wrs-vxworks               powerpc64-wrs-vxworks
arm-linux-androideabi             powerpc64le-unknown-linux-gnu
arm-unknown-linux-gnueabi         powerpc64le-unknown-linux-musl
arm-unknown-linux-gnueabihf       riscv32i-unknown-none-elf
arm-unknown-linux-musleabi        riscv32imac-unknown-none-elf
arm-unknown-linux-musleabihf      riscv32imc-unknown-none-elf
armebv7r-none-eabi                riscv64gc-unknown-linux-gnu
armebv7r-none-eabihf              riscv64gc-unknown-none-elf
armv4t-unknown-linux-gnueabi      riscv64imac-unknown-none-elf
armv5te-unknown-linux-gnueabi     s390x-unknown-linux-gnu
armv5te-unknown-linux-musleabi    sparc-unknown-linux-gnu
armv6-unknown-freebsd             sparc64-unknown-linux-gnu
armv6-unknown-netbsd-eabihf       sparc64-unknown-netbsd
armv7-linux-androideabi           sparc64-unknown-openbsd
armv7-unknown-cloudabi-eabihf     sparcv9-sun-solaris
armv7-unknown-freebsd             thumbv6m-none-eabi
armv7-unknown-linux-gnueabi       thumbv7a-pc-windows-msvc
armv7-unknown-linux-gnueabihf     thumbv7em-none-eabi
armv7-unknown-linux-musleabi      thumbv7em-none-eabihf
armv7-unknown-linux-musleabihf    thumbv7m-none-eabi
armv7-unknown-netbsd-eabihf       thumbv7neon-linux-androideabi
armv7-wrs-vxworks-eabihf          thumbv7neon-unknown-linux-gnueabihf
armv7a-none-eabi                  thumbv7neon-unknown-linux-musleabihf
armv7a-none-eabihf                thumbv8m.base-none-eabi
armv7r-none-eabi                  thumbv8m.main-none-eabi
armv7r-none-eabihf                thumbv8m.main-none-eabihf
asmjs-unknown-emscripten          wasm32-unknown-emscripten
hexagon-unknown-linux-musl        wasm32-unknown-unknown
i586-pc-windows-msvc              wasm32-wasi
i586-unknown-linux-gnu            x86_64-apple-darwin
i586-unknown-linux-musl           x86_64-fortanix-unknown-sgx
i686-apple-darwin                 x86_64-fuchsia
i686-linux-android                x86_64-linux-android
i686-pc-windows-gnu               x86_64-linux-kernel
i686-pc-windows-msvc              x86_64-pc-solaris
i686-unknown-cloudabi             x86_64-pc-windows-gnu
i686-unknown-freebsd              x86_64-pc-windows-msvc
i686-unknown-haiku                x86_64-rumprun-netbsd
i686-unknown-linux-gnu            x86_64-sun-solaris
i686-unknown-linux-musl           x86_64-unknown-cloudabi
i686-unknown-netbsd               x86_64-unknown-dragonfly
i686-unknown-openbsd              x86_64-unknown-freebsd
i686-unknown-uefi                 x86_64-unknown-haiku
i686-uwp-windows-gnu              x86_64-unknown-hermit
i686-uwp-windows-msvc             x86_64-unknown-hermit-kernel
i686-wrs-vxworks                  x86_64-unknown-illumos
mips-unknown-linux-gnu            x86_64-unknown-l4re-uclibc
mips-unknown-linux-musl           x86_64-unknown-linux-gnu
mips-unknown-linux-uclibc         x86_64-unknown-linux-gnux32
mips64-unknown-linux-gnuabi64     x86_64-unknown-linux-musl
mips64-unknown-linux-muslabi64    x86_64-unknown-netbsd
mips64el-unknown-linux-gnuabi64   x86_64-unknown-openbsd
mips64el-unknown-linux-muslabi64  x86_64-unknown-redox
mipsel-unknown-linux-gnu          x86_64-unknown-uefi
mipsel-unknown-linux-musl         x86_64-uwp-windows-gnu
mipsel-unknown-linux-uclibc       x86_64-uwp-windows-msvc
mipsisa32r6-unknown-linux-gnu     x86_64-wrs-vxworks
```

You can print the specification of one of these targets using the following command:

``` console
$ rustc +nightly -Z unstable-options --print target-spec-json --target thumbv7m-none-eabi
{
  "abi-blacklist": [
    "stdcall",
    "fastcall",
    "vectorcall",
    "thiscall",
    "win64",
    "sysv64"
  ],
  "arch": "arm",
  "data-layout": "e-m:e-p:32:32-i64:64-v128:64:128-a:0:32-n32-S64",
  "emit-debug-gdb-scripts": false,
  "env": "",
  "executables": true,
  "is-builtin": true,
  "linker": "arm-none-eabi-gcc",
  "linker-flavor": "gcc",
  "llvm-target": "thumbv7m-none-eabi",
  "max-atomic-width": 32,
  "os": "none",
  "panic-strategy": "abort",
  "relocation-model": "static",
  "target-c-int-width": "32",
  "target-endian": "little",
  "target-pointer-width": "32",
  "vendor": ""
}
```

If none of these built-in targets seems appropriate for your target system, you'll have to create a
custom target by writing your own target specification file in JSON format which is described in the
[next section][custom-target].

[custom-target]: ./custom-target.md

## `rust-std` component

For some of the built-in target the Rust team distributes `rust-std` components via `rustup`. This
component is a collection of pre-compiled crates like `core` and `std`, and it's required for cross
compilation.

You can find the list of targets that have a `rust-std` component available via `rustup` by running
the following command:

``` console
$ rustup target list | column
aarch64-apple-ios                       mipsel-unknown-linux-musl
aarch64-fuchsia                         nvptx64-nvidia-cuda
aarch64-linux-android                   powerpc-unknown-linux-gnu
aarch64-pc-windows-msvc                 powerpc64-unknown-linux-gnu
aarch64-unknown-linux-gnu               powerpc64le-unknown-linux-gnu
aarch64-unknown-linux-musl              riscv32i-unknown-none-elf
aarch64-unknown-none                    riscv32imac-unknown-none-elf
aarch64-unknown-none-softfloat          riscv32imc-unknown-none-elf
arm-linux-androideabi                   riscv64gc-unknown-linux-gnu
arm-unknown-linux-gnueabi               riscv64gc-unknown-none-elf
arm-unknown-linux-gnueabihf             riscv64imac-unknown-none-elf
arm-unknown-linux-musleabi              s390x-unknown-linux-gnu
arm-unknown-linux-musleabihf            sparc64-unknown-linux-gnu
armebv7r-none-eabi                      sparcv9-sun-solaris
armebv7r-none-eabihf                    thumbv6m-none-eabi
armv5te-unknown-linux-gnueabi           thumbv7em-none-eabi
armv5te-unknown-linux-musleabi          thumbv7em-none-eabihf
armv7-linux-androideabi                 thumbv7m-none-eabi
armv7-unknown-linux-gnueabi             thumbv7neon-linux-androideabi
armv7-unknown-linux-gnueabihf           thumbv7neon-unknown-linux-gnueabihf
armv7-unknown-linux-musleabi            thumbv8m.base-none-eabi
armv7-unknown-linux-musleabihf          thumbv8m.main-none-eabi
armv7a-none-eabi                        thumbv8m.main-none-eabihf
armv7r-none-eabi                        wasm32-unknown-emscripten
armv7r-none-eabihf                      wasm32-unknown-unknown
asmjs-unknown-emscripten                wasm32-wasi
i586-pc-windows-msvc                    x86_64-apple-darwin
i586-unknown-linux-gnu                  x86_64-apple-ios
i586-unknown-linux-musl                 x86_64-fortanix-unknown-sgx
i686-linux-android                      x86_64-fuchsia
i686-pc-windows-gnu                     x86_64-linux-android
i686-pc-windows-msvc                    x86_64-pc-windows-gnu
i686-unknown-freebsd                    x86_64-pc-windows-msvc
i686-unknown-linux-gnu                  x86_64-rumprun-netbsd
i686-unknown-linux-musl                 x86_64-sun-solaris
mips-unknown-linux-gnu                  x86_64-unknown-cloudabi
mips-unknown-linux-musl                 x86_64-unknown-freebsd
mips64-unknown-linux-gnuabi64           x86_64-unknown-linux-gnu (default)
mips64-unknown-linux-muslabi64          x86_64-unknown-linux-gnux32
mips64el-unknown-linux-gnuabi64         x86_64-unknown-linux-musl
mips64el-unknown-linux-muslabi64        x86_64-unknown-netbsd
mipsel-unknown-linux-gnu                x86_64-unknown-redox
```

If there's no `rust-std` component for your target, or you are using a custom target, then you'll
have to use a nightly toolchain to build the standard library. See the next page about [building for
custom targets][use-target-file].

[use-target-file]: ./custom-target.md#use-the-target-file
[xargo]: https://github.com/japaric/xargo
