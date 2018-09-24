set -euxo pipefail

main() {
    local tag=$(git ls-remote --tags --refs --exit-code https://github.com/rust-lang-nursery/mdbook \
                    | cut -d/ -f3 \
                    | grep -E '^v[0.1.0-9.]+$' \
                    | sort --version-sort \
                    | tail -n1)
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- --git rust-lang-nursery/mdbook --tag $tag

    rustup target add thumbv7m-none-eabi

    # install arm-none-eabi-gcc
    mkdir gcc

    curl -L https://developer.arm.com/-/media/Files/downloads/gnu-rm/7-2018q2/gcc-arm-none-eabi-7-2018-q2-update-linux.tar.bz2?revision=bc2c96c0-14b5-4bb4-9f18-bceb4050fee7?product=GNU%20Arm%20Embedded%20Toolchain,64-bit,,Linux,7-2018-q2-update | tar --strip-components=1 -C gcc -xj

    mkdir qemu
    curl -L https://github.com/japaric/qemu-bin/blob/master/14.04/qemu-system-arm-2.12.0 > qemu/qemu-system-arm

    rustup component add llvm-tools-preview

    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- --git rust-embedded/cargo-binutils --tag v0.1.4

    pip install linkchecker --user
}

main
