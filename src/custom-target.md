# Creating a custom target

If a custom target triple is not available for your platform, you must create a custom target file
that describes your target to rustc.

Keep in mind that it is required to use a nightly compiler to build the core library, which must be
done for a target unknown to rustc.

## Decide on a target triple

Many targets already have a known triple used to describe them, typically in the form
ARCH-VENDOR-SYS-ABI. You should aim to use the same triple that [LLVM uses][llvm-target-triple];
however, it may differ if you need to specify additional information to Rust that LLVM does not know
about. Although the triple is technically only for human use, it's important for it to be unique and
descriptive especially if the target will be upstreamed in the future.

The ARCH part is typically just the architecture name, except in the case of 32-bit ARM. For
example, you would probably use x86_64 for those processors, but specify the exact ARM architecture
version. Typical values might be `armv7`, `armv5te`, or `thumbv7neon`. Take a look at the names of
the [built-in targets][built-in-target] for inspiration.

The VENDOR part is optional, and describes the manufacturer. Omitting this field is the same as
using `unknown`.

The SYS part describes the OS that is used. Typical values include `win32`, `linux`, and `darwin`
for desktop platforms. `none` is used for bare-metal usage.

The ABI part describes how the process starts up. `eabi` is used for bare metal, while `gnu` is used
for glibc, `musl` for musl, etc.

Now that you have a target triple, create a file with the name of the triple and a `.json`
extension. For example, a file describing `armv7a-none-eabi` would have the filename
`armv7a-none-eabi.json`.

[llvm-target-triple]: https://clang.llvm.org/docs/CrossCompilation.html#target-triple

## Fill the target file

The target file must be valid JSON. There are two places where its contents are described:
[`Target`], where every field is mandatory, and [`TargetOptions`], where every field is optional.
**All underscores are replaced with hyphens**.

The recommended way is to base your target file on the specification of a built-in target that's
similar to your target system, then tweak it to match the properties of your target system. To do
so, use the command
`rustc +nightly -Z unstable-options --print target-spec-json --target $SOME_SIMILAR_TARGET`, using
[a target that's already built into the compiler][built-in-target].

You can pretty much copy that output into your file. Start with a few modifications:

- Remove `"is-builtin": true`
- Fill `llvm-target` with [the triple that LLVM expects][llvm-target-triple]
- Decide on a panicking strategy. A bare metal implementation will likely use
  `"panic-strategy": "abort"`. If you decide not to `abort` on panicking, even if you [tell Cargo
  to][aborting-on-panic], you must define an [eh_personality] function.
- Configure atomics. Pick the first option that describes your target:
  - I have a single-core processor, no threads, no interrupts, or any way for multiple things to be
    happening in parallel: if you are **sure** that is the case, such as WASM (for now), you may set
    `"singlethread": true`. This will configure LLVM to convert all atomic operations to use their
    single threaded counterparts.
  - I have native atomic operations: set `max-atomic-width` to the biggest type in bits that your
    target can operate on atomically. For example, many ARM cores have 32-bit atomic operations. You
    may set `"max-atomic-width": 32` in that case.
  - I have no native atomic operations, but I can emulate them myself: set `max-atomic-width` to the
    highest number of bits that you can emulate up to 64, then implement all of the
    [atomic][libcalls-atomic] and [sync][libcalls-atomic] functions expected by LLVM as
    `#[no_mangle] unsafe extern "C"`. These functions have been standardized by gcc, so the [gcc
    documentation][gcc-sync] may have more notes. Missing functions will cause a linker error, while
    incorrectly implemented functions will possibly cause UB.
  - I have no native atomic operations: you'll have to do some unsafe work to manually ensure
    synchronization in your code. You must set `"max-atomic-width": 0`.
- Change the linker if integrating with an existing toolchain. For example, if you're using a
  toolchain that uses a custom build of gcc, set `"linker-flavor": "gcc"` and `linker` to the
  command name of your linker. If you require additional linker arguments, use `pre-link-args` and
  `post-link-args` as so:
  ``` json
  "pre-link-args": {
      "gcc": [
          "-Wl,--as-needed",
          "-Wl,-z,noexecstack",
          "-m64"
      ]
  },
  "post-link-args": {
      "gcc": [
          "-Wl,--allow-multiple-definition",
          "-Wl,--start-group,-lc,-lm,-lgcc,-lstdc++,-lsupc++,--end-group"
      ]
  }
  ```
  Ensure that the linker type is the key within `link-args`.
- Configure LLVM features. Run `llc -march=ARCH -mattr=help` where ARCH is the base architecture
  (not including the version in the case of ARM) to list the available features and their
  descriptions. **If your target requires strict memory alignment access (e.g. `armv5te`), make sure
  that you enable `strict-align`**. To enable a feature, place a plus before it. Likewise, to
  disable a feature, place a minus before it. Features should be comma separated like so:
  `"features": "+soft-float,+neon`. Note that this may not be necessary if LLVM knows enough about
  your target based on the provided triple and CPU.
- Configure the CPU that LLVM uses if you know it. This will enable CPU-specific optimizations and
  features. At the top of the output of the command in the last step, there is a list of known CPUs.
  If you know that you will be targeting a specific CPU, you may set it in the `cpu` field in the
  JSON target file.

[`target`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_target/spec/struct.Target.html
[`targetoptions`]:
  https://doc.rust-lang.org/nightly/nightly-rustc/rustc_target/spec/struct.TargetOptions.html
[aborting-on-panic]:
  https://doc.rust-lang.org/edition-guide/rust-2018/error-handling-and-panics/aborting-on-panic.html
[built-in-target]: ./compiler-support.md#built-in-target
[eh_personality]: ./smallest-no-std.md#eh_personality
[libcalls-atomic]: http://llvm.org/docs/Atomics.html#libcalls-atomic
[libcalls-sync]: http://llvm.org/docs/Atomics.html#libcalls-sync
[gcc-sync]: https://gcc.gnu.org/onlinedocs/gcc/_005f_005fsync-Builtins.html

## Use the target file

Once you have a target specification file, you may refer to it by its path or by its name (i.e.
excluding `.json`) if it is in the current directory or in `$RUST_TARGET_PATH`.

Verify that it is readable by rustc:

``` sh
❱ rustc --print cfg --target foo.json # or just foo if in the current directory
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

Now, you finally get to use it! Many resources have been recommending [`xargo`] or [`cargo-xbuild`].
However, its successor, cargo's `build-std` feature, has received a lot of work recently and has
quickly reached feature parity with the other options. As such, this guide will only cover that
option.

Start with a bare minimum [`no_std` program][no_std-program]. Now, run
`cargo build -Z build-std=core --target foo.json`, again using the above rules about referencing the
path. Hopefully, you should now have a binary in the target directory.

You may optionally configure cargo to always use your target. See the recommendations at the end of
the page about [the smallest `no_std` program][no_std-program]. However, you'll currently have to
use the flag `-Z build-std=core` as that option is unstable.

[`xargo`]: https://github.com/japaric/xargo
[`cargo-xbuild`]: https://github.com/rust-osdev/cargo-xbuild
[no_std-program]: ./smallest-no-std.md

### Build additional built-in crates

When using cargo's `build-std` feature, you can choose which crates to compile in. By default, when
only passing `-Z build-std`, `std`, `core`, and `alloc` are compiled. However, you may want to
exclude `std` when compiling for bare-metal. To do so, specify the crated you'd like after
`build-std`. For example, to include `core` and `alloc`, pass `-Z build-std=core,alloc`.

## Troubleshooting

### language item required, but not found: `eh_personality`

Either add `"panic-strategy": "abort"` to your target file, or define an [eh_personality] function.

### undefined reference to `__sync_val_compare_and_swap_#`

Rust thinks that your target has atomic instructions, but LLVM doesn't. Go back to the step about
[configuring atomics][fill-target-file]. You will need to reduce the number in `max-atomic-width`.
See [#58500] for more details.

[fill-target-file]: #fill-the-target-file
[#58500]: https://github.com/rust-lang/rust/issues/58500

### could not find `sync` in `alloc`

Similar to the above case, Rust doesn't think that you have atomics. You must implement them
yourself or [tell Rust that you have atomic instructions][fill-target-file].

### multiple definition of `__(something)`

You're likely linking your Rust program with code built from another language, and the other
language includes compiler built-ins that Rust also creates. To fix this, you'll need to tell your
linker to allow multiple definitions. If using gcc, you may add:

``` json
"post-link-args": {
    "gcc": [
        "-Wl,--allow-multiple-definition"
    ]
}
```

### error adding symbols: file format not recognized

Switch to cargo's `build-std` feature and update your compiler. This [was a bug][#8239] introduced
for a few compiler builds that tried to pass in internal Rust object to an external linker.

[#8239]: https://github.com/rust-lang/cargo/issues/8239
