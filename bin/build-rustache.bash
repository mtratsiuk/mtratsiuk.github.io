#!/usr/bin/env bash

set -eu
set -o pipefail

cd "$(dirname "$0")"/..

TARGET=x86_64-unknown-linux-gnu

cd rustache \
    && cargo test \
    && cargo build --release --target=$TARGET \
    && cp target/$TARGET/release/rustache ../bin/rustache \
    && rm -rf target \
    && cd ..
