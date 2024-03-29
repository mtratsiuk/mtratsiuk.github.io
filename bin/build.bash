#!/usr/bin/env bash

set -eu
set -o pipefail

cd "$(dirname "$0")"/..

mkdir -p ./build
cp -a ./src/assets/. ./build

./bin/rustache --in=./src --out=./build/index.html
