# Guide for silicon vendors to enable Rust support for their SoCs

## Introduction

Rust has emerged as a powerful and safety-focused programming language, gaining
traction among embedded developers. Silicon vendors who wish to enable Rust
support for their System-on-Chip (SoC) products can benefit from this trend by
attracting a growing community of Rust developers.

This guide aims to help silicon vendors enable Rust support, either
independently or by empowering third-party developers. It outlines the
essential resources, tasks, and priorities required to foster a robust Rust
ecosystem around their System-on-Chip (SoC).

**Note:** For assistance with strategy in engaging with the community, we
recommend reaching out to the Rust Embedded Working Group (REWG) leads. They
can provide valuable insights and support to help you navigate the process
effectively.

## Essential resources

### Documentation

Detailed documentation is essential for effective development and debugging. It
enables developers to comprehend the System-on-Chip (SoC), including its memory
map, peripherals, interrupt handling, low-power modes, etc. Ensure that the
documentation covers all hardware aspects comprehensively, from register-level
details to system-level interactions. The documentation should be publicly
available; in cases where public availability is not feasible, any
non-disclosure agreement (NDA) must permit the publication of open-source code
derived from it.

### Register description files

Register description files are used to generate Peripheral Access Crates
(PACs). The most common format for these files is SVD
([System View Description](https://open-cmsis-pack.github.io/svd-spec)). Rust
developers have often encountered issues with SVD files, so it is crucial to
provide clear contact information for reporting any discrepancies or problems.
Up-to-date SVD files ensure that the community can collaborate effectively to
resolve issues and improve the quality of the PACs.

### Flash Algorithms

[Flash Algorithms](https://open-cmsis-pack.github.io/Open-CMSIS-Pack-Spec/main/html/flashAlgorithm.html)
are integrated with debugging tools like [probe-rs](https://probe.rs). They
facilitate and speed up firmware programming and debugging, streamlining
development workflows. Providing well-supported FlashAlgos will enhance the
integration with these tools and improve the overall developer experience.

### Vendor tooling

Some System-on-Chip (SoC) devices require custom tools for generating images or
flashing them onto the device. It is beneficial to provide these tools in an
open-source manner, fostering community contributions and accelerating
ecosystem growth. Open-sourcing vendor tooling enables third-party developers
to extend and enhance the toolchain, ensuring improved compatibility with the
broader Embedded Rust ecosystem.

### Contact information

Providing contact information is vital for addressing maintainer queries and
issues related to register description files or other resources. The use of a
public issue tracking system (like GitHub Issues) for reporting and tracking
problems might help. Actively engage with the community through forums,
discussions, and updates to build trust and collaboration.

## Maintaining PAC and HAL crates

Peripheral Access Crates (PACs) and Hardware Abstraction Layer (HAL) crates are
at the core of enabling Rust support.

### Generate and maintain PACs

Multiple tools such as [svd2rust](https://crates.io/crates/svd2rust),
[chiptool](https://github.com/embassy-rs/chiptool),
[raltool](https://github.com/imxrt-rs/imxrt-ral/tree/master/raltool), and
[svd2pac](https://github.com/Infineon/svd2pac) automate the generation of PACs
from register description files. Each tool has its strengths, and selecting the
right one depends on the requirements and the complexity of the hardware.

### Develop and maintain HAL crates

Implement [embedded-hal](https://crates.io/crates/embedded-hal),
[embedded-hal-async](https://crates.io/crates/embedded-hal-async), and
[embedded-io](https://crates.io/crates/embedded-io) traits in your HAL crates.
Adhering to these traits ensures compatibility across the Embedded Rust
ecosystem, enhancing interoperability. It is an essential goal that HALs use
Rust code rather than wrapping existing C code. An incremental porting
strategy, where all core functionality is implemented in Rust, but C with Rust
bindings is used for complex drivers, is acceptable, allowing for gradual
adoption and community contributions.

Start with essential peripherals (clock, timer, GPIO) and expand progressively
(I2C, SPI, UART, etc.) based on community feedback. Release early and often to
engage the community and gather valuable insights for further development.

### Common recommendations

- Ensure that crates are compatible with `no_std` environments, which are
  common in embedded systems without an operating system. Functionality that
  needs `alloc` or `std` can be included when gated with Cargo
  [features](https://doc.rust-lang.org/cargo/reference/features.html).
- Make your crates available on [crates.io](https://crates.io) to maximize
  visibility and ease of use for developers.
- Use [semantic versioning](https://semver.org) to maintain consistency and
  predictability in your releases.
- Prefer licenses like Apache 2.0 and MIT for their permissive nature, which
  encourages broader adoption and collaboration.

### Issue tracking

Effective issue tracking is crucial for maintaining a healthy and collaborative
ecosystem. Discuss triaging, labeling, and community involvement in issue
resolution. Implement transparent processes for:

- Triage and prioritize issues based on severity and impact.
- Use labels to categorize issues (e.g., bugs, feature requests).
- Encourage community members to contribute to resolving issues by providing
  feedback or submitting pull requests (PRs).

### Facilitate debugging and testing

The Embedded Rust ecosystem offers various tools used for debugging
and testing, with [probe-rs](https://probe.rs) being one of the most widely
used. [probe-rs](https://probe.rs) supports a wide range
of target architectures, debug interfaces, and debug probe protocols.
Combined with debug-based facilities like
[defmt-rtt](https://crates.io/crates/defmt-rtt), which provide logging
capabilities for embedded systems, these tools form a robust foundation for
development.

Thorough testing ensures hardware-software reliability, and leveraging these
tools can significantly enhance development workflows.

## Nice-to-have features for enhanced ecosystem support

### Examples

Including some basic examples as part of the HAL is essential for helping
developers get started. These examples should demonstrate key functionalities,
such as initializing peripherals or handling interrupts. They serve as
practical starting points and learning aids.

### BSP (Board Support Package) crates

BSP crates are relevant when you need to provide board-specific configurations
and initializations. Unlike HALs, which focus on hardware abstraction, BSPs
handle the integration of multiple components for a specific board. Separation
in BSP and HAL crates offers a layered approach, making it easier for developers
to build applications targeting a particular hardware board.

### Project templates

Project templates are boilerplate code structures that provide a starting point
for new projects. They include prevalent configurations, dependencies, and
setup steps, saving developers time and reducing the learning curve. Examples
of project templates include bare-metal (using the HAL without any framework),
Embassy, RTIC, and others.

### Integration with popular IDEs and tools

Offer guides on setting up development environments for Embedded Rust projects
with popular tools such as:

- [rust-analyzer](https://rust-analyzer.github.io): for Rust syntax
  highlighting and error checking.
- [probe-rs](https://probe.rs): for flashing and debugging firmware.
- [defmt](https://crates.io/crates/defmt): a logging framework optimized for
  embedded systems, including a test harness called
  [defmt-test](https://crates.io/crates/defmt-test).

Providing setup instructions for these tools will help developers integrate
them into their workflows, enhancing productivity and collaboration.

## Suggested flow for adding SoC Support

- A preliminary requirement of this flow is that the Rust toolchain includes
  a [target](https://doc.rust-lang.org/rustc/platform-support.html) that
  matches System-on-Chip (SoC). If this not the case the solution can be as simple as adding a
  [custom target](https://doc.rust-lang.org/rustc/targets/custom.html) or as
  difficult as adding support for the underlying architecture to
  [LLVM](https://llvm.org).
- Before starting from scratch, check if any existing community efforts for
  already exist (e.g. checking on
  [awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust)
  or joining the
  [Rust Embedded Matrix room](https://matrix.to/#/#rust-embedded:matrix.org)).
  This could save significant development time.
- Ensure that your target is supported by [probe-rs](https://probe.rs). The
  ability to debug using SWD or JTAG is highly beneficial. Support for flashing
  programming can be added with a Flash Algorithm (e.g. from a CMSIS-Pack).
- Generate Peripheral Access Crates (PACs) from register description files with
  SVD (System View Description) being the most common and preferred format.
  Alternatives include extracting the register descriptions from PDF datasheets
  or C header files, but this much more labor-intensive.
- Create a minimal project containing the PAC and/or an empty Hardware
  Abstraction Layer (HAL). The goal is to get a minimal working binary that
  either blinks an LED or sends messages through
  [defmt-rtt](https://crates.io/crates/defmt-rtt) using only the PAC crate or
  with a minimal HAL. This will require a linker script and exercise the
  availability to flash and debug programs. Additional crates for core
  registers and peripheral, or startup code and interrupt handling will also be
  required (see [Cortex-M](https://github.com/rust-embedded/cortex-m) or
  [RISC-V](https://github.com/rust-embedded/riscv)).
- Add core functionality in HAL: clocks, timers, interrupts. Verify the
  accuracy of timers and interrupts with external tools like a logic analyzer
  or an oscilloscope.
- Progressively add drivers for other peripherals (GPIO, I2C, SPI, UART, etc.)
  implementing standard Rust Embedded traits
  ([embedded-hal](https://crates.io/crates/embedded-hal),
  [embedded-hal-async](https://crates.io/crates/embedded-hal-async),
  [embedded-io](https://crates.io/crates/embedded-io)).

## Conclusion

Enabling Rust support for your SoC opens the door to a vibrant community of
developers who value safety, performance, and reliability. By providing
essential resources, maintaining high-quality PACs and HAL crates, and
fostering a supportive ecosystem, you empower both internal teams and
third-party developers to unlock the full potential of your hardware.

As the Rust embedded ecosystem continues to grow, embracing these practices
positions your company at the forefront of this movement, attracting developers
passionate about building robust and innovative systems. Encourage ongoing
engagement with the Rust community to stay updated on best practices and tools,
ensuring your System-on-Chip (SoC) remains a preferred choice for Rust
developers.

By following this guide, you can create a comprehensive and supportive
environment that not only enables Rust support but also nurtures a thriving
developer ecosystem around your products.
