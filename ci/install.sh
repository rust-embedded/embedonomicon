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

    rustup component add llvm-tools-preview

    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- --git rust-embedded/cargo-binutils --tag v0.1.2

    pip install linkchecker --user
}

main
