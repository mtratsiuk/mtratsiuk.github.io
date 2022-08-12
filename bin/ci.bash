#!/usr/bin/env bash

set -eu
set -o pipefail

cd "$(dirname "$0")"/..

./bin/build.bash

# TODO: purge CF cache
