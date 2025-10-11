# Why don't we initialize `.data` and `.bss` using Rust

Earlier versions of this book initialized the `.data` and `.bss` sections using Rust code.
This has proven to have questionable soundness, and the recommended method of 
performing the initialization of these sections nowadays relies on assembly. 

This chapter discusses the reasons that led to the decision of various crates like 
[cortex-m-rt](https://crates.io/crates/cortex-m-rt) and [riscv-rt](https://crates.io/crates/riscv-rt) 
to migrate to performing assembly initialization of these sections. There are 
[a](https://github.com/rust-embedded/cortex-m-rt/issues/300) 
[decent](https://github.com/rust-embedded/embedonomicon/issues/69)
[number](https://rust-lang.zulipchat.com/#narrow/stream/136281-t-lang.2Fwg-unsafe-code-guidelines/topic/The.20least.20incorrect.20init.20code.20.3A\))
[of](https://github.com/rust-lang/unsafe-code-guidelines/issues/259)
[threads](https://github.com/rust-embedded/wg/issues/771) 
where the soundness of such code has been questioned. We will summarize 
them in this chapter.

The original code used for global data initialization in Rust in this book is listed
as follows:

``` rust
{{#include ../ci/main/rt-unsound/src/lib.rs:1:32}}
```

Five `extern "C"` variables are declared to reference specific memory locations.
Our linker script defines each symbol, so we do not need to worry about their 
exact placement.

## Pointer proventace

To initialize the `.bss` section, we take the address of `_sbss` `u8` variable, 
which points to the start of the `.bss` section. Then we write an arbitrary 
amount of data to its location. `_sbss` is declared as an `u8` variables, and 
the pointer provenance rules only allow us to write an amount of data that fits 
within the allocation of our `_sbss` variable. Despite that, we are writing past 
the single byte (as far as Rust is aware, a single byte is allocated at this 
address) up until we hit the location of the `_ebss`.

There is a separate issue in which we actually have an `_ebss` variable that is 
pointing one byte outside of the `.bss` section. In specific implementations, 
accessing this byte might not even be possible if the `.bss` section exhausted 
the available memory. Ideally `_ebss` needs to be declared as a ZST. And by 
extension, because the `.bss` section can be empty, `_sbss` should also be a 
ZST, because in this case `_sbss` would also fall outside of the region reserved 
for the `.bss`.

## Aliasing

Another potential problem with the code above is aliasing. Consider our linker 
script.

``` text
{{#include ../ci/main/rt-unsound/link.x:36:48}}
```

The following situations can occur:
- `_sbss` might be located at the same address as the first variable in the `.bss`
section, assuming that the section is not empty.
- `_ebss` will be located at the same address as `_sdata`, and by extension, it 
will also be located at the same address as the first variable in the `.data` 
section.
- If the `.bss` section is empty, both `_sbss` and `_ebss` will alias each other.
- If the `.data` section is empty, both `_sdata` and `_edata` will alias each other.

Rust does not allow to have more than one variable to be located at the same address 
(with ZSTs being a key exception). But even if it did, we are using these variables 
to write the whole global memory area, which effectively is mutably aliasing all 
global data defined in the program.

## Abstract machine initialization

Another question is whether it is safe to enter any Rust code before the Rust 
abstract machine has been fully initialized. Can we rely on Rust not using any 
of the global memory while it is not yet initialized? The answer to this question 
is not clear (or does not seem clear to the author of the section at the time of 
this writing).

## More potential provenance issues

A clever reader might have seen how we compute the offset between `_ebss` and `_sbss` and thought,
couldn't we instad use the [`offset_from`](https://doc.rust-lang.org/std/primitive.pointer.html#method.offset_from) 
method of a pointer?

The problem with this approach, however, is that, as we mentioned above, both `_ebss`
and `_sbss` belong to different allocations, so they do not share the same pointer 
provenance. This is true even if they both are aliased and happen to fall at the 
same address (i.e. when the `.bss` section is empty).

Running Miri on this [Rust Playground Snippet](https://play.rust-lang.org/?version=stable&mode=release&edition=2024&gist=3225a585752704d9c58b1842e0fc5307)
shows the undefined behavior.

## Ok, but it works, doesn't it?

Yes. While the code provided at the beginning of this chapter does produce the 
right behavior as of Rust 1.89, the problem is that **we cannot rely on this behavior
being preserved in future releases**, or even in the optimizer doing something 
funky in the future.

That is why, overall, the recommendation of this books is to **not** perform the initialization 
using Rust code for this purpose.
