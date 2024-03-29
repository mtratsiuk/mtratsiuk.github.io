#!/usr/bin/env bash

set -eu
set -o pipefail

cd "$(dirname "$0")"/..

cd rustache \
    && cargo test \
    && cargo build --release \
    && cp target/release/mt-rustache ../bin/rustache \
    && rm -rf target \
    && cd ..
