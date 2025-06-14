# Guide for Silicon Vendors to Enable Rust Support for Their SoCs

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
recommend reaching out to the Embedded Rust Working Group (REWG) leads. They
can provide valuable insights and support to help you navigate the process
effectively.

## Essential Resources

### Documentation

Detailed documentation is essential for effective development and debugging. It
enables developers to comprehend the System-on-Chip (SoC), including its memory
map, peripherals, interrupt handling, low-power modes, etc. Ensure that the
documentation covers all hardware aspects comprehensively, from register-level
details to system-level interactions. The documentation should be publicly
available; in cases where public availability is not feasible, any
non-disclosure agreement (NDA) must permit the publication of open-source code
derived from it.

### Register Description Files

Register description files are used to generate Peripheral Access Crates
(PACs). The most common format for these files is SVD
([System View Description](https://open-cmsis-pack.github.io/svd-spec)). Rust
developers have often encountered issues with SVD files, so it is crucial to
provide clear contact information for reporting any discrepancies or problems.
Up-to-date SVD files ensure that the community can collaborate effectively to
resolve issues and improve the quality of the PACs.

### Flash Algorithms

[Flash Algorithms](https://open-cmsis-pack.github.io/Open-CMSIS-Pack-Spec/main/html/flashAlgorithm.html)
are integrated with debugging tools like `probe-rs`. They facilitate and speed
up firmware programming and debugging, streamlining development workflows.
Providing well-supported FlashAlgos will enhance the integration with these
tools and improve the overall developer experience.

### Vendor Tooling

Some System-on-Chip (SoC) devices require custom tools for generating images or
flashing them onto the device. It is beneficial to provide these tools in an
open-source manner, fostering community contributions and accelerating
ecosystem growth. Open-sourcing vendor tooling enables third-party developers
to extend and enhance the toolchain, ensuring improved compatibility with the
broader Embedded Rust ecosystem.

### Contact Information

Providing contact information is vital for addressing maintainer queries and
issues related to register description files or other resources. The use of a
public issue tracking system (like GitHub Issues) for reporting and tracking
problems might help. Actively engage with the community through forums,
discussions, and updates to build trust and collaboration.

## Maintaining PAC and HAL Crates

Peripheral Access Crates (PACs) and Hardware Abstraction Layer (HAL) crates are
at the core of enabling Rust support.

### Generate and Maintain PACs

Multiple tools such as `svd2rust`, `chiptool`, `raltool`, and `svd2pac`
automate the generation of PACs from register description files. Each tool has
its strengths, and selecting the right one depends on the requirements and the
complexity of the hardware.

### Develop and Maintain HAL Crates

Implement `embedded-hal` and `embedded-hal-async` traits in your HAL crates.
Adhering to these traits ensures compatibility across the Embedded Rust
ecosystem, enhancing interoperability. It is an essential goal that HALs use
Rust code rather than wrapping existing C code. An incremental porting
strategy, where Rust is used for all core functionality, but C with Rust
bindings is used for complex drivers, is acceptable, allowing for gradual
adoption and community contributions.

Start with essential peripherals (clock, timer, GPIO) and expand progressively
(I2C, SPI, UART, etc.) based on community feedback. Release early and often to
engage the community and gather valuable insights for further development.

### Common Recommendations

- Ensure that crates are compatible with `no_std` environments, which are
  common in embedded systems without an operating system. Functionality that
  needs `alloc` or `std` can be included when gated with Cargo "features."
- Make your crates available on [crates.io](https://crates.io/) to maximize
  visibility and ease of use for developers.
- Use semantic versioning to maintain consistency and predictability in your
  releases.
- Prefer licenses like Apache 2.0 and MIT for their permissive nature, which
  encourages broader adoption and collaboration.

### Issue Tracking

Effective issue tracking is crucial for maintaining a healthy and collaborative
ecosystem. Discuss triaging, labeling, and community involvement in issue
resolution. Implement transparent processes for:

- Triage and prioritize issues based on severity and impact.
- Use labels to categorize issues (e.g., bugs, feature requests).
- Encourage community members to contribute to resolving issues by providing
  feedback or submitting pull requests (PRs).

### Facilitate Debugging and Testing

The use of `probe-rs` is prevalent in the Embedded Rust ecosystem for debugging
and testing. Combined with debug-based facilities like `defmt-rtt`, which
offers logging capabilities for embedded systems, the Embedded Rust ecosystem
has developed numerous tools. `probe-rs` supports a wide range of target
architectures, debug interfaces, and debug probe protocols.

Thorough testing ensures hardware-software reliability, and leveraging these
tools can significantly enhance development workflows.

## Nice-to-Have Features for Enhanced Ecosystem Support

### Examples

Including some basic examples as part of the HAL is essential for helping
developers get started. These examples should demonstrate key functionalities,
such as initializing peripherals or handling interrupts. They serve as
practical starting points and learning aids.

### BSP (Board Support Package) Crates

BSP crates are relevant when you need to provide board-specific configurations
and initializations. Unlike HALs, which focus on hardware abstraction, BSPs
handle the integration of multiple components for a specific board. Having both
BSP and HAL crates offers a layered approach, making it easier for developers
to build applications targeting a particular hardware board.

### Project Templates

Project templates are boilerplate code structures that provide a starting point
for new projects. They include prevalent configurations, dependencies, and
setup steps, saving developers time and reducing the learning curve. Examples
of project templates include bare-metal (using the HAL without any framework),
Embassy, RTIC, and others.

### Integration with Popular IDEs and Tools

Offer guides on setting up development environments for Embedded Rust projects
with popular tools such as:

- `rust-analyzer`: for Rust syntax highlighting and error checking.
- `probe-rs`: for flashing and debugging firmware.
- `defmt`: a logging framework optimized for embedded systems.
- `defmt-test`: testing utilities for `defmt`.

Providing setup instructions for these tools will help developers integrate
them into their workflows, enhancing productivity and collaboration.

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
