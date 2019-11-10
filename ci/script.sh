set -euxo pipefail

main() {
    # build the book and check that it has no dead links
    mdbook build

    linkchecker book

    # now check this as a directory of the bookshelf
    rm -rf shelf
    mkdir shelf
    mv book shelf
    linkchecker shelf

    mv shelf/book .
    rmdir shelf

    # test the instructions at different stages
    cd ci

    # # smallest-no-std
    pushd smallest-no-std

    # check that it builds
    cargo rustc -- --emit=obj

    # check that the output is an empty binary
    # NOTE(-b) ignore change in whitespace
    diff -b app.size \
         <(cargo size --bin app)

    # check presence of the `rust_begin_unwind` symbol
    diff app.o.nm \
         <(cargo nm -- target/thumbv7m-none-eabi/debug/deps/app-*.o | grep '[0-9]* [^N] ')

    edition_check

    popd

    # # memory-layout
    pushd memory-layout

    # check that the Reset symbol is there
    diff app.text.objdump \
         <(cargo objdump --bin app -- -d -no-show-raw-insn -no-leading-addr)

    # check that the reset vector is there and has the right address
    diff app.vector_table.objdump \
         <(cargo objdump --bin app -- -s -section .vector_table)

    qemu_check target/thumbv7m-none-eabi/debug/app

    edition_check

    popd

    # # main
    pushd main

    # check that the disassembly matches
    pushd app
    diff app.objdump \
         <(cargo objdump --bin app -- -d -no-show-raw-insn -no-leading-addr)
    # disabled because of rust-lang/rust#53964
    # edition_check
    popd

    # check that it builds
    pushd app2
    cargo build
    edition_check
    popd

    pushd app3
    cargo build
    edition_check
    popd

    # NOTE(nightly) this will require nightly until core::arch::arm::udf is stabilized
    if [ $TRAVIS_RUST_VERSION = nightly ]; then
        pushd app4
        cargo build
        qemu_check target/thumbv7m-none-eabi/debug/app
        edition_check
        popd
    fi

    popd

    # # exception handling
    # NOTE(nightly) this will require nightly until core::arch::arm::udf is stabilized
    if [ $TRAVIS_RUST_VERSION = nightly ]; then
        pushd exceptions

        # check that the disassembly matches
        pushd app
        diff app.objdump \
             <(cargo objdump --bin app --release -- -d -no-show-raw-insn -print-imm-hex -no-leading-addr)
        diff app.vector_table.objdump \
             <(cargo objdump --bin app --release -- -s -j .vector_table)
        edition_check
        popd

        # check that it builds
        pushd app2
        cargo build
        edition_check
        popd

        popd
    fi

    # # Assembly on stable
    pushd asm

    # check that the disassembly matches
    pushd app
    diff release.objdump \
         <(cargo objdump --bin app --release -- -d -no-show-raw-insn -print-imm-hex -no-leading-addr)
    diff release.vector_table \
         <(cargo objdump --bin app --release -- -s -j .vector_table)
    edition_check
    popd

    # check that the binary blob is up to date
    pushd rt2
    arm-none-eabi-as -march=armv7-m asm.s -o asm.o
    ar crs librt.a asm.o
    diff librt.objdump \
         <(arm-none-eabi-objdump -Cd librt.a)
    popd

    # check that the disassembly matches
    pushd app2
    diff release.objdump \
         <(cargo objdump --bin app --release -- -d -no-show-raw-insn -print-imm-hex -no-leading-addr)
    diff release.vector_table \
         <(cargo objdump --bin app --release -- -s -j .vector_table)
    edition_check
    popd

    popd

    # # Logging with symbols
    pushd logging

    # check that the ~output~ and disassembly matches
    # the output won't exactly match because addresses of static variables won't
    # remain the same when the toolchain is updated. Instead we'll that the
    # printed address is contained in the output of `cargo objdump -- -t`
    pushd app
    cargo run > dev.out
    cargo objdump --bin app -- -t | grep '\.rodata\s*0*1\b' > dev.objdump
    for address in $(cat dev.out); do
        grep ${address#0x} dev.objdump
    done

    cargo run --release > release.out
    cargo objdump --bin app --release -- -t | grep '\.rodata\s*0*1\b' > release.objdump
    for address in $(cat release.out); do
        grep ${address#0x} release.objdump
    done

    # sanity check the committed files
    git checkout dev.out
    git checkout dev.objdump
    for address in $(cat dev.out); do
        grep ${address#0x} dev.objdump
    done

    git checkout release.out
    git checkout release.objdump
    for address in $(cat release.out); do
        grep ${address#0x} release.objdump
    done
    edition_check
    popd

    # check that the output and disassembly matches
    pushd app2
    diff dev.out \
         <(cargo run | xxd -p)
    diff dev.objdump \
         <(cargo objdump --bin app -- -t | grep '\.log')
    edition_check
    popd

    # check that the output and disassembly matches
    pushd app3
    diff dev.out \
         <(cargo run | xxd -p)
    diff dev.objdump \
         <(cargo objdump --bin app -- -t | grep '\.log')
    edition_check
    popd

    # check that the output and disassembly matches
    pushd app4
    diff dev.out \
         <(cargo run | xxd -p)
    diff dev.objdump \
         <(cargo objdump --bin app -- -t | grep '\.log')
    edition_check
    popd

    popd

    # # Logging with symbols
    pushd singleton

    pushd app
    diff dev.out \
         <(cargo run | xxd -p)
    diff dev.objdump \
         <(cargo objdump --bin app -- -t | grep '\.log')
    diff release.objdump \
         <(cargo objdump --bin app --release -- -t | grep LOGGER)
    edition_check
    popd

    popd

    # # DMA
    # NOTE(nightly) this will require nightly until core::pin is stabilized (1.33)
    if [ $TRAVIS_RUST_VERSION = nightly ]; then
        pushd dma
        cargo build --examples
        popd
    fi
}

# checks that 2018 idioms are being used
edition_check() {
    RUSTFLAGS="-D rust_2018_compatibility -D rust_2018_idioms" cargo check
}

# checks that QEMU doesn't crash and that it produces no error messages
qemu_check() {
    qemu-system-arm \
        -cpu cortex-m3 \
        -machine lm3s6965evb \
        -nographic \
        -kernel $1 \
        >.stdout 2>.stderr &

    local pid=$!
    sleep 3
    # check that: process is still running && stdout is empty && stderr is empty
    kill -9 $pid && ! [ -s .stdout ] && ! [ -s .stderr ] || \
            ( cat .stdout && cat .stderr && exit 1)
    rm .stdout .stderr
}

# don't run this on successful merges
if [[ $TRAVIS_BRANCH != main || $TRAVIS_PULL_REQUEST != false ]]; then
    main
fi
