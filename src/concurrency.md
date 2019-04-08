# Concurrency

This section discusses `no_std` concurrency as usually found on
microcontrollers, and memory safe patterns for sharing memory with / between
interrupt handlers. The focus of this text is on uses of `unsafe` code that are
memory safety rather than building safe abstractions.

> **NOTE:** Unlike other chapters, this text has been written assuming that the
> reader is *not* familiar with the interrupt mechanism commonly found in
> microcontrollers. The motivation is making this text accessible to more people
> who then can audit our `unsafe` code.

# Interrupts

In bare metal systems, systems without an OS (operating system), usually the
only form of concurrency available are *hardware* interrupts. An interrupt
is a preemption mechanism that works as follows: when an *interrupt signal*
arrives the processor suspends the execution of the current subroutine, (maybe)
saves some registers (the current state of the program) to the stack and then
jumps to another subroutine called the *interrupt handler*. When the processor
returns from the interrupt handler, it restores the registers that it previously
saved on the stack (if any) and then resumes the subroutine that was
interrupted. (If you are familiar with POSIX signal handling, the semantics are
pretty much the same)

Interrupt signals usually come from peripherals and are fired *asynchronously*.
Some examples of interrupt signals are: a counter reaching zero, an input pin
changing its electrical / logical state, and the arrival of a new byte of data.
In some multi-core devices a core can send an interrupt signal to a different
core.

How the processor locates the right interrupt handler to execute depends on the
architecture. In the ARM Cortex-M architecture, there's one handler per
interrupt signal and there's a table somewhere in memory that holds function
pointers to all interrupt handlers. Each interrupt is given an index in this
table. For example, a timer interrupt could be interrupt #0 and an input pin
interrupt could be interrupt #1. If we were to depict this as Rust code it would
look as follows:

``` rust
// `link_section` places this in some known memory location
#[link_section = ".interrupt_table"]
static INTERRUPT_TABLE: [extern "C" fn(); 32] = [
    // entry 0: timer 0
    on_timer0_interrupt,

    // entry 1: pin 0
    on_pin0_interrupt,

    // .. 30 more entries ..
];

// provided by the application author
extern "C" fn on_timer0_interrupt() {
    // ..
}

extern "C" fn on_pin0_interrupt() {
    // ..
}
```

In another common interrupt model all interrupts signals map to the *same*
interrupt handler (subroutine) and there's a hardware register that the software
has to read when it enters the handler to figure out which interrupt signal
triggered the interrupt. In this text, we'll focus on the ARM Cortex-M
architecture which follows the one handler per interrupt signal model.

## Interrupt handling API

The most basic interrupt handling API lets the programmer *statically* register
a function for each interrupt handler *only once*. On top of this basic API
it's possible to implement APIs to *dynamically* register closures as interrupt
handlers. In this text we'll focus on the former, simpler API.

To illustrate this kind of API let's look at the [`cortex-m-rt`] crate (v0.6.7).
It provides two attributes to statically register interrupts: `#[exception]` and
`#[interrupt]`. The former is for device agnostic interrupts, whose number and
names are the same for all Cortex-M devices; the latter is for device specific
interrupts, whose number and names vary per device / vendor. We'll stick to the
device agnostic interrupts ("exceptions") in our examples.

[`cortex-m-quickstart`]: https://github.com/rust-embedded/cortex-m-quickstart
[`cortex-m-rt`]:  https://crates.io/crates/cortex-m-rt/0.6.7

The following example showcases the system timer (`SysTick`) interrupt, which
fires periodically. The interrupt is handled using the `SysTick` handler
(function), which prints a dot to the console.

> **NOTE:** The code for the following example and all other examples can be
> found in the `ci/concurrency` directory at the root of [this repository].

[this repository]: https://github.com/rust-embedded/embedonomicon

``` rust
{{#include ../ci/concurrency/examples/systick.rs}}```

If you are not familiar with embedded / Cortex-M programs the most important
thing to point note here is that the function marked with the `entry` attribute
is the entry point of the user program. When the device (re)boots (e.g. it's
first powered) the "runtime" (the `cortex-m-rt` crate) initializes `static`
variables (the content of RAM is random on power on) and then calls the user
program entry point. As the user program is the only process running it is not
allowed to end / exit; this is enforced in the signature of the `entry`
function: `fn() -> !` -- a divergent function can't return.

You can run this example on an x86 machine using QEMU. Make sure you have
`qemu-system-arm` installed and run the following command

``` console
$ cargo run --example systick
(..)
     Running `qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel target/thumbv7m-none-eabi/debug/examples/systick`
.................
```

## `static` variables: what is safe and what's not

As interrupt handlers have their own (call) stack they can't refer to (access)
local variables in `main` or in functions called by `main`. The only way `main`
and an interrupt handler can share state is through `static` variables, which
have statically known addresses.

To really drive this point I find it useful to visualize the call stack of the
program in the presence of interrupts. Consider the following example:

``` rust
#[entry]
fn main() -> ! {

    loop {
        {
            let x = 42;
            foo();
        }

        {
            let w = 66;
            bar();
        }
    }
}

fn foo() {
   let y = 24;

   // ..
}

fn bar() {
    let z = 33;

    // ..

    foo();

    // ..
}

#[exception]
fn SysTick() {
    // can't access `x` or `y` because their addresses are not statically known
}
```

If we take snapshots of the call stack every time the `SysTick` interrupt
handler is called we'll observe something like this:

``` text
                                                          +---------+
                                                          | SysTick |
                                                          |         |
            +---------+            +---------+            +#########+
            | SysTick |            | SysTick |            |   foo   |
            |         |            |         |            | y = 24  |
            +#########+            +#########+            +---------+
            |   foo   |            |   bar   |            |   bar   |
            | y = 24  |            | z = 33  |            | z = 33  |
            +---------+            +---------+            +---------+
            |   main  |            |   main  |            |   main  |
            | x = 42  |            | w = 66  |            | w = 66  |
            +---------+            +---------+            +---------+
              t = 1ms                t = 2ms                t = 3ms
```

From the call stack `SysTick` looks like a normal function since it's contiguous
in memory to `main` and the functions called from it. However, that's not the
case: `SysTick` is invoked asynchronously. At time `t = 1ms` `SysTick` could, in
theory, access `y` since it's in the previous stack frame; however, at time `t =
2ms` `y` doesn't exist; and at time `t = 3ms` `y` exists but has a different
location in memory (address).

I hope that explains why `SysTick` can't safely access the stack frames that
belong to `main`.

Let's now go over all the `unsafe` and safe ways in which `main` and interrupt
handlers can share state (memory). We'll start assuming the program will run on
a single core device, then we'll revisit our safe patterns in the context of a
multi-core device.

### `static mut`

Unsynchronized access to `static mut` variables is undefined behavior (UB). The
compiler *will* mis-optimize all those accesses.

Consider the following *unsound* program:

``` rust
{{#include ../ci/concurrency/examples/static-mut.rs}}```

This program compiles: both `main` and `SysTick` can refer to the static
variable `X`, which has a known, fixed location in memory. However, the program
is mis-optimized to the following machine code:

``` armasm
00000400 <main>:
 400:   bf00            nop
 402:   e7fd            b.n     400 <main>

00000404 <SysTick>:
 404:   bf00            nop
 406:   4770            bx      lr
```

As you can see all accesses to `X` were optimized away changing the intended
semantics.

### Volatile

Using volatile operations to access `static mut` variables does *not* prevent
UB. Volatile operations will prevent the compiler from mis-optimizing accesses
to the variables but they don't help with torn reads and writes which lead to
UB.

``` rust
{{#include ../ci/concurrency/examples/volatile.rs}}```

In this program the interrupt handler could preempt the 2-step write operation
that changes `X` from variant `A` to variant `B` (or vice versa) mid way. If
that happens the handler could observe `X` having the value
`0x0000_0000_0000_0000` or `0xffff_ffff_ffff_ffff`, neither of which are valid
values for the enum.

Let me say that again: *Relying only on volatile operations for memory safety
is likely wrong*. The only semantics that volatile operations provide are:
"tell the compiler to not remove this operation, or merge it with another
operation" and "tell the compiler to not reorder this operation with respect to
other *volatile* operations"; neither is directly related to synchronized
access to memory.

### Atomics

Accessing atomics stored in `static` variables is memory safe. If you are
building abstractions like channels on top of them (which likely will require
`unsafe` code to access some shared buffer) make sure you use the right
`Ordering` or your abstraction will be unsound.

Here's an example of using a static variable for synchronization (a delay in
this case).

> **NOTE:** not all embedded targets have atomic CAS instructions in their ISA.
> MSP430 and ARMv6-M are prime examples. API like `AtomicUsize.fetch_add` is not
> available in `core` for those targets.

``` rust
static X: AtomicBool = AtomicBool::new(false);

#[entry]
fn main() -> ! {
    // omitted: configuring and enabling the `SysTick` interrupt

    // wait until `SysTick` returns before starting the main logic
    while !X.load(Ordering::Relaxed) {}

    loop {
        // main logic
    }
}

#[exception]
fn SysTick() {
    X.store(true, Ordering::Relaxed);
}
```

### State and re-entrancy

A common pattern in embedded C is to use a `static` variable to preserve state
between invocations of an interrupt handler.

``` c
void handler() {
    static int counter = 0;

    counter += 1;

    // ..
}
```

This makes the function non-reentrant, meaning that calling this function from
itself, from `main` or an interrupt handler is UB (it breaks mutable aliasing
rules).

We can make this C pattern safe in Rust if we make the non-reentrant function
`unsafe` to call or impossible to call. `cortex-m-rt` v0.5.x supports this
pattern and uses the latter approach to prevent calling non-reentrant functions
from safe code.

Consider this example:

``` rust
{{#include ../ci/concurrency/examples/state.rs}}```

The `#[exception]` attribute performs the following source-level transformation:

``` rust
#[link_name = "SysTick"] // places this function in the vector table
fn randomly_generated_identifier() {
    let COUNTER: &mut u64 = unsafe {
        static mut COUNTER: u64 = 0;

        &mut COUNTER
    };

    // user code
    *COUNTER += 1;

    // ..
}
```

Placing the `static mut` variable inside a block makes it impossible to create
more references to it from user code.

This transformation ensures that the software can't call the interrupt handler
from safe code, but could the hardware invoke the interrupt handler in a way
that breaks memory safety? The answer is: *it depends*, on the target
architecture.

In the ARM Cortex-M architecture once an instance of an interrupt handler starts
another one won't start until the first one ends (if the same interrupt signal
arrives again it is withheld). On the other hand, in the ARM Cortex-R
architecture there's a single handler for all interrupts; receiving two
different interrupt signals can cause the handler (function) to be invoked twice
and that would break the memory safety of the source level transformation we
presented above.

### Critical sections

When it's necessary to share state between `main` and an interrupt handler a
critical section can be used to synchronize access. The simplest critical
section implementation consists of temporarily disabling *all* interrupts while
`main` accesses the shared `static` variable. Example below:

``` rust
{{#include ../ci/concurrency/examples/cs1.rs}}```

Note the use of the `"memory"` clobber; this acts as a compiler barrier that
prevents the compiler from reordering the operation on `COUNTER` to outside the
critical section. It's also important to *not* access `COUNTER` in `main`
outside a critical section; thus references to `COUNTER` should not escape the
critical section. With these two restrictions in place, the mutable reference to
`COUNTER` created in `SysTick` is guaranteed to be unique for the whole
execution of the handler.

Disabling all the interrupt is not the only way to create a critical section;
other ways include masking interrupts (disabling one or a subset of all
interrupts) and increasing the running priority (see next section).

Masking interrupts to create a critical section deserves an example because it
doesn't use inline `asm!` and thus requires explicit compiler barriers
(`atomic::compiler_fence`) for memory safety.

``` rust
{{#include ../ci/concurrency/examples/cs2.rs}}```

The code is very similar to the one that disabled all interrupts except for the
start and end of the critical section, which now include a `compiler_fence`
(compiler barrier).

### Priorities

Architectures like ARM Cortex-M allow interrupt prioritization, meaning that an
interrupt that's given high priority can preempt a lower priority interrupt
handler. Priorities must be considered when sharing state between interrupt
handlers.

When two interrupt handlers, say `A` and `B`, have the *same* priority no
preemption can occur. Meaning that when signals for both interrupts arrive
around the same time then the handlers will be executed sequentially: that is
first `A` and then `B`, or vice versa. In this scenario, both handlers can
access the same `static mut` variable *without* using a critical section; each
handler will "take turns" at getting exclusive access (`&mut-`) to the static
variable. Example below.

``` rust
{{#include ../ci/concurrency/examples/coop.rs}}```

When two interrupt handlers have *different* priorities then one can preempt
the other. Safely sharing state between these two interrupts requires a critical
section in the lower priority handler -- just like in the case of `main` and an
interrupt handler. However, one more constraint is required: the priority of the
interrupts must remain fixed at runtime; reversing the priorities at runtime,
for example, would result in a data race.

The following example showcases safe state sharing between two interrupt
handlers using a priority-based critical section.

``` rust
{{#include ../ci/concurrency/examples/cs3.rs}}```

### Runtime initialization

A common need in embedded Rust programs is moving, at runtime, a value from
`main` into an interrupt handler. This can be accomplished at zero cost by
enforcing sequential access to `static mut` variables.

``` rust
{{#include ../ci/concurrency/examples/init.rs}}```

In this pattern is important to disable interrupts before yielding control to
the user program and enforcing that the end user initializes all the
uninitialized static variables before interrupts are re-enabled. Failure to do
so would result in interrupt handlers observing uninitialized static variables.

## Redefining `Send` and `Sync`

The core / standard library defines these two marker traits as:

> `Sync`: types for which it is safe to share references between threads.
>
> `Send`: types that can be transferred across thread boundaries

Threads are an OS abstraction so they don't exist "out of the box" in bare metal
context, though they can be implemented on top of interrupts. We'll broaden the
definition of these two marker traits to include bare metal code:

- `Sync`: types for which it is safe to share references between *execution
  contexts*.

- `Send`: types that can be transferred between *execution contexts*.

An interrupt handler is an execution context independent of the `main` function,
which can be seen as the "bottom" execution context. An OS thread is also an
execution context. Each execution context has its own (call) stack and operates
independently of other execution contexts though they can share state.

Broadening the definitions of these marker traits does not change the rules
around `static` variables. They must still hold values that implement the `Sync`
trait. Atomics implement `Sync` so they are valid to place in `static` variables
in bare metal context.

Let's now revisit the safe patterns we described before and see where the `Sync`
and `Send` bounds need to be enforced for safety.

### State

``` rust
#[exception]
fn SysTick() {
    static mut X: Type = Type::new();
}
```

Does `Type` need to satisfy `Sync` or `Send`? `X` is effectively owned by the
`SysTick` interrupt and not shared with any other execution context so neither
bound is required for this pattern.

### Critical section

We can abstract the "disable all interrupts" critical section pattern into a
`Mutex` type.

``` rust
{{#include ../ci/concurrency/examples/mutex.rs}}```

Here we use a `CriticalSection` token to prevent references escaping the
critical section / closure (see the lifetime constraints in `Mutex.borrow`).

It's important to note that a `Mutex.borrow_mut` method with no additional
runtime checks would be unsound as it would let the end user break Rust aliasing
rules:

``` rust
#[exception]
fn SysTick() {
    interrupt::free(|cs| {
        // both `counter` and `alias` refer to the same memory location
        let counter: &mut u64 = COUNTER.borrow_mut(cs);
        let alias: &mut u64 = COUNTER.borrow_mut(cs);
    });
}
```

Changing the signature of `borrow_mut` to `fn<'cs>(&self, &'cs mut
CriticalSection) -> &'cs mut T` does *not* help because it's possible to nest
calls to `interrupt::free`.

``` rust
#[exception]
fn SysTick() {
    interrupt::free(|cs: &mut CriticalSection| {
        let counter: &mut u64 = COUNTER.borrow_mut(cs);

        // let alias: &mut u64 = COUNTER.borrow_mut(cs);
        //~^ ERROR: `cs` already mutably borrowed

        interrupt::free(|cs2: &mut CriticalSection| {
            // this breaks aliasing rules
            let alias: &mut u64 = COUNTER.borrow_mut(cs2);
        });
    });
}
```

As for the bounds required on the value of type `T` protected by the `Mutex`:
`T` must implement the `Send` trait because a `Mutex` can be used as a channel
to move values from `main` to an interrupt handler. See below:

``` rust
struct Thing {
    _state: (),
}

static CHANNEL: Mutex<RefCell<Option<Thing>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    interrupt::free(|cs| {
        let channel = CHANNEL.borrow(cs);

        *channel.borrow_mut() = Some(Thing::new());
    });

    loop {
        asm::nop();
    }
}

#[exception]
fn SysTick() {
    interrupt::free(|cs| {
        let channel = CHANNEL.borrow(cs);
        let maybe_thing = channel.borrow_mut().take();
        if let Some(thing) = mabye_thing {
            // `thing` has been moved into the interrupt handler
        }
    });
}
```

So the `Sync` implementation must look like this:

``` rust
unsafe impl<T> Sync for Mutex<T> where T: Send {}
```

This constraint applies to all types of critical sections.

### Runtime initialization

For the pattern of moving values from `main` to an interrupt handler this is
clearly a "send" operation so the moved value must implement the `Send` trait.
We won't give an example of an abstraction for that pattern in this text but any
such abstraction must enforce at compile time that values to be moved implement
the `Send` trait.

## Multi-core

So far we have discussed single core devices. Let's see how having multiple
cores affects the memory safety of the abstractions and patterns we have
covered.

### `Mutex: !Sync`

The `Mutex` abstraction we created and that disables interrupts to create a
critical section is unsound in multi-core context. The reason is that the
critical section doesn't prevent *other* cores from making progress so if more
than one core gets a reference to the data behind the `Mutex` all accesses
become data races.

Here an example where we assume a dual-core device and a framework that lets you
write bare-metal multi-core in a single source file.

``` rust
// THIS PROGRAM IS UNSOUND!

// single memory location visible to both cores
static COUNTER: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

// runs on the first core
#[core(0)]
#[entry]
fn main() -> ! {
    loop {
        interrupt::free(|cs| {
            let counter = COUNTER.borrow(cs);

            counter.set(counter.get() + 1);
        });
    }
}

// runs on the second core
#[core(1)]
#[entry]
fn main() -> ! {
    loop {
        interrupt::free(|cs| {
            let counter = COUNTER.borrow(cs);

            counter.set(counter.get() * 2);
        });
    }
}
```

Here each core accesses the `COUNTER` variable in their `main` context in an
unsynchronized manner; this is undefined behavior.

The problem with `Mutex` is not the critical section that uses; it's the fact
that it can be stored in a `static` variable making accessible to all cores.
Thus in multi-core context the `Mutex` abstraction should not implement the
`Sync` trait.

Critical sections based on interrupt masking *can* be used safely on
architectures / devices where it's possible to assign a *single* core to an
interrupt and any core can mask that interrupt, provided that scoping is
enforced somehow. Here's an example:

``` rust
static mut COUNTER: u64 = 0;

// runs on the first core
// priority = 2
#[core(0)]
#[exception]
fn SysTick() {
    // exclusive access to `COUNTER`
    let counter: &mut u64 = unsafe { &mut COUNTER };

    *counte += 1;
}

// initialized in the second core's `main` function using the runtime
// initialization pattern
static mut SYST: MaybeUninit<SYST> = MaybeUninit::ununitialized();

// runs on the second core
// priority = 1
#[core(1)]
#[exception]
fn SVCall() {
    // `SYST` is owned by this core / interrupt
    let syst = unsafe { &mut *SYST.as_mut_ptr() };

    // start of critical section: disable the `SysTick` interrupt
    syst.disable_interrupt();

    atomic::compiler_fence(Ordering::SeqCst);

    // `SysTick` can not preempt this block
    {
        let counter: &mut u64 = unsafe { &mut COUNTER };

        *counter += 1;
    }

    atomic::compiler_fence(Ordering::SeqCst);

    // end of critical section: re-enable the `SysTick` interrupt
    syst.enable_interrupt();
}
```

### Atomics

Atomics are safe to use in multi-core context provided that memory barrier
instructions are inserted where appropriate. If you are using the correct
`Ordering` then the compiler will insert the required barriers for you. Critical
sections based on atomics, AKA spinlocks, are memory safe to use on multi-core
devices though they can deadlock.

``` rust
// spin = "0.5.0"
use spin::Mutex;

static COUNTER: Mutex<u64> = Mutex::new(0);

// runs on the first core
#[core(0)]
#[entry]
fn main() -> ! {
    loop {
        *COUNTER.lock() += 1;
    }
}

// runs on the second core
#[core(1)]
#[entry]
fn main() -> ! {
    loop {
        *COUNTER.lock() *= 2;
    }
}
```

### State

The stateful interrupt handler pattern remains safe if and only if the target
architecture / device supports assigning a handler to a single core and the
program has been configured to not share stateful interrupts between cores --
that is cores should *not* execute the exact same handler when the corresponding
signal arrives.

### Runtime initialization

As the runtime initialization pattern is used to initialize the "state" of
interrupt handlers so all the additional constraints required for multi-core
memory safety of the State pattern are also required here.
