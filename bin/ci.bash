#!/usr/bin/env bash

set -eu
set -o pipefail

cd "$(dirname "$0")"/..

./bin/e2e.bash
./bin/build.bash
./bin/purge-cache.bash

cp CNAME ./build/CNAME
