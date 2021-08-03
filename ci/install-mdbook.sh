#!/usr/bin/env sh

set -eo pipefail


tag=$(git ls-remote --tags --refs --exit-code https://github.com/rust-lang/mdbook \
          | grep -Eo 'v[0-9.]+$' \
          | sort --version-sort \
          | tail -n1)

curl -LSfs https://japaric.github.io/trust/install.sh | \
    sh -s -- --git rust-lang/mdbook --tag $tag
