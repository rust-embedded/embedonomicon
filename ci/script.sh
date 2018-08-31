set -euxo pipefail

main() {
    # build the book and check that it has no dead links
    mdbook build

    linkchecker book

    # test the instructions at different stages
    cd ci

    # # smallest-no-std
    pushd smallest-no-std

    # check that it builds
    cargo rustc -- --emit=obj

    # check that the output is an empty binary
    diff app.size \
         <(cargo size --bin app)

    # check presence of the `rust_begin_unwind` symbol
    diff app.o.nm \
         <(cargo nm -- target/thumbv7m-none-eabi/debug/deps/app-*.o | grep '[0-9]* [^n] ')

    popd

    # # memory-layout
    pushd memory-layout

    # check that the Reset symbol is there
    diff app.text.objdump \
         <(cargo objdump --bin app -- -d -no-show-raw-insn)

    # check that the reset vector is there and has the right address
    diff app.vector_table.objdump \
         <(cargo objdump --bin app -- -s -section .vector_table)

    qemu_check target/thumbv7m-none-eabi/debug/app

    popd

    # # main
    pushd main

    # check that the disassembly matches
    pushd app
    diff app.objdump \
         <(cargo objdump --bin app -- -d -no-show-raw-insn)
    popd

    # check that it builds
    pushd app2
    cargo build
    popd

    pushd app3
    cargo build
    popd

    pushd app4
    cargo build
    qemu_check target/thumbv7m-none-eabi/debug/app
    popd

    popd

    # # exception handling
    pushd exceptions

    # check that the disassembly matches
    pushd app
    diff app.vector_table.objdump \
         <(cargo objdump --bin app --release -- -s -j .vector_table)
    popd

    # check that it builds
    pushd app2
    cargo build
    popd

    popd
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
