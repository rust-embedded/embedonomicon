# A note on compiler support

This book makes use of a built-in *compiler* target, the `thumbv7m-none-eabi`, for which the Rust
team distributes a `rust-std` component, a pre-compiled version of the `core` crate.

If you want to attempt replicating the contents of this book for a different target architecture you
need to take into account the different levels of support that Rust provides for (compilation)
targets.

## LLVM support

As of Rust 1.28 official Rust compiler, `rustc`, uses LLVM for (machine) code generation. The
minimal level of support Rust provides for an architecture is having its LLVM backend enabled in
`rustc`. You can see all the architectures that `rustc` supports, through LLVM, by running the
following command:

``` console
$ # you need to have `cargo-binutils` installed to run this command
$ cargo objdump -- -version
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

If LLVM supports the architecture you are interested in but `rustc` is built with the backend
disabled (which is the case of AVR as of Rust 1.28) then you will need to modify the Rust source
enabling it. The first two commits of PR [rust-lang/rust#52787] give you an idea of the required
changes.

[rust-lang/rust#52787]: https://github.com/rust-lang/rust/pull/52787

On the other hand, if LLVM doesn't support the architecture but a fork of LLVM does you will have to
built `rustc` against the fork. The Rust build system allows this and in principle should just
require changing the `llvm` submodule to point to the fork.

If your target architecture is only supported by some vendor provided GCC you have the option of
using [`mrustc`], an unofficial Rust compiler, to translate your Rust program into C code and then
compile that using GCC.

[`mrustc`]: https://github.com/thepowersgang/mrustc

## Built-in target

A compilation target is more than just its architecture. Each target has a [specification]
associated to it that describes its architecture, its operating system, the default linker among
other things.

[specification]: https://github.com/rust-lang/rfcs/blob/master/text/0131-target-specification.md

The Rust compiler knows about several targets. These are said to be *built into* the compiler and
can be listed by running the following command:

``` console
$ rustc --print target-list | column
aarch64-fuchsia                 mips64el-unknown-linux-gnuabi64
aarch64-linux-android           mipsel-unknown-linux-gnu
aarch64-unknown-cloudabi        mipsel-unknown-linux-musl
aarch64-unknown-freebsd         mipsel-unknown-linux-uclibc
aarch64-unknown-linux-gnu       msp430-none-elf
aarch64-unknown-linux-musl      powerpc-unknown-linux-gnu
aarch64-unknown-openbsd         powerpc-unknown-linux-gnuspe
arm-linux-androideabi           powerpc-unknown-netbsd
arm-unknown-linux-gnueabi       powerpc64-unknown-linux-gnu
arm-unknown-linux-gnueabihf     powerpc64le-unknown-linux-gnu
arm-unknown-linux-musleabi      powerpc64le-unknown-linux-musl
arm-unknown-linux-musleabihf    s390x-unknown-linux-gnu
armebv7r-none-eabihf            sparc-unknown-linux-gnu
armv4t-unknown-linux-gnueabi    sparc64-unknown-linux-gnu
armv5te-unknown-linux-gnueabi   sparc64-unknown-netbsd
armv5te-unknown-linux-musleabi  sparcv9-sun-solaris
armv6-unknown-netbsd-eabihf     thumbv6m-none-eabi
armv7-linux-androideabi         thumbv7em-none-eabi
armv7-unknown-cloudabi-eabihf   thumbv7em-none-eabihf
armv7-unknown-linux-gnueabihf   thumbv7m-none-eabi
armv7-unknown-linux-musleabihf  wasm32-experimental-emscripten
armv7-unknown-netbsd-eabihf     wasm32-unknown-emscripten
asmjs-unknown-emscripten        wasm32-unknown-unknown
i586-pc-windows-msvc            x86_64-apple-darwin
i586-unknown-linux-gnu          x86_64-fuchsia
i586-unknown-linux-musl         x86_64-linux-android
i686-apple-darwin               x86_64-pc-windows-gnu
i686-linux-android              x86_64-pc-windows-msvc
i686-pc-windows-gnu             x86_64-rumprun-netbsd
i686-pc-windows-msvc            x86_64-sun-solaris
i686-unknown-cloudabi           x86_64-unknown-bitrig
i686-unknown-dragonfly          x86_64-unknown-cloudabi
i686-unknown-freebsd            x86_64-unknown-dragonfly
i686-unknown-haiku              x86_64-unknown-freebsd
i686-unknown-linux-gnu          x86_64-unknown-haiku
i686-unknown-linux-musl         x86_64-unknown-l4re-uclibc
i686-unknown-netbsd             x86_64-unknown-linux-gnu
i686-unknown-openbsd            x86_64-unknown-linux-gnux32
mips-unknown-linux-gnu          x86_64-unknown-linux-musl
mips-unknown-linux-musl         x86_64-unknown-netbsd
mips-unknown-linux-uclibc       x86_64-unknown-openbsd
mips64-unknown-linux-gnuabi64   x86_64-unknown-redox
```

You can print the specification of any of these targets using the following command:

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

If none of these built-in targets seems appropriate for your target system you'll have to create a
custom target by writing a target specification file. A target specification file is a JSON file
that contains the specification of a compilation target. The output of the `rustc --print
target-spec-json` command from before is the target specification in JSON format.

The recommended way to write a target specification file is to dump the specification of a built-in
target that's similar to your target system into a file and then tweak it to match the properties of
your target system. As of Rust 1.28, there's no up to date documentation on what each of the fields
of a target specification mean, other than [the compiler source code].

[the compiler source code]: https://github.com/rust-lang/rust/blob/1.27.2/src/librustc_target/spec/mod.rs#L376-L400

Once you have a target specification file you can refer to it by its path or by its name if its in
the current directory or in `$RUST_TARGET_PATH`.

``` console
$ rustc +nightly -Z unstable-options --print target-spec-json \
      --target thumbv7m-none-eabi \
      > foo.json

$ rustc --print cfg --target foo.json # or just --target foo
debug_assertions
target_arch="arm"
target_endian="little"
target_env=""
target_feature="mclass"
target_feature="v7"
target_has_atomic="16"
target_has_atomic="32"
target_has_atomic="8"
target_has_atomic="cas"
target_has_atomic="ptr"
target_os="none"
target_pointer_width="32"
target_vendor=""
```

## `rust-std` component

For some of the built-in target the Rust team distributes `rust-std` components via `rustup`. This
component is a collection of pre-compiled crates like `core` and `std`, and it's required for
cross compilation.

You can find the list of targets that have a `rust-std` component available via `rustup` by running
the following command:

``` console
$ rustup target list | column
aarch64-apple-ios                       mips64-unknown-linux-gnuabi64
aarch64-linux-android                   mips64el-unknown-linux-gnuabi64
aarch64-unknown-fuchsia                 mipsel-unknown-linux-gnu
aarch64-unknown-linux-gnu               mipsel-unknown-linux-musl
aarch64-unknown-linux-musl              powerpc-unknown-linux-gnu
arm-linux-androideabi                   powerpc64-unknown-linux-gnu
arm-unknown-linux-gnueabi               powerpc64le-unknown-linux-gnu
arm-unknown-linux-gnueabihf             s390x-unknown-linux-gnu
arm-unknown-linux-musleabi              sparc64-unknown-linux-gnu
arm-unknown-linux-musleabihf            sparcv9-sun-solaris
armv5te-unknown-linux-gnueabi           thumbv6m-none-eabi
armv5te-unknown-linux-musleabi          thumbv7em-none-eabi
armv7-apple-ios                         thumbv7em-none-eabihf
armv7-linux-androideabi                 thumbv7m-none-eabi
armv7-unknown-linux-gnueabihf           wasm32-unknown-emscripten
armv7-unknown-linux-musleabihf          wasm32-unknown-unknown
armv7s-apple-ios                        x86_64-apple-darwin
asmjs-unknown-emscripten                x86_64-apple-ios
i386-apple-ios                          x86_64-linux-android
i586-pc-windows-msvc                    x86_64-pc-windows-gnu
i586-unknown-linux-gnu                  x86_64-pc-windows-msvc
i586-unknown-linux-musl                 x86_64-rumprun-netbsd
i686-apple-darwin                       x86_64-sun-solaris
i686-linux-android                      x86_64-unknown-cloudabi
i686-pc-windows-gnu                     x86_64-unknown-freebsd
i686-pc-windows-msvc                    x86_64-unknown-fuchsia
i686-unknown-freebsd                    x86_64-unknown-linux-gnu (default)
i686-unknown-linux-gnu                  x86_64-unknown-linux-gnux32
i686-unknown-linux-musl                 x86_64-unknown-linux-musl
mips-unknown-linux-gnu                  x86_64-unknown-netbsd
mips-unknown-linux-musl                 x86_64-unknown-redox
```

If there's no `rust-std` component for your target or you are using a custom target then you'll have
to use a tool like [Xargo] to have Cargo compile the `core` crate on the fly. Note that Xargo
requires a nightly toolchain; the long term plan is to upstream Xargo's functionality into Cargo
and eventually have that functionality available on stable.

[Xargo]: https://github.com/japaric/xargo
