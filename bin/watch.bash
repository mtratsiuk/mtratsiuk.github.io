#!/usr/bin/env bash

set -eu
set -o pipefail

cd "$(dirname "$0")"/..

last_hash=""

inotifywait -m ./src -e create -e delete -e move -e modify |
    while read directory action file; do
        current_hash=$(md5sum "$directory$file")

        if [ "$current_hash" != "$last_hash" ]; then
            ./bin/build.bash
        fi

        last_hash="$current_hash"
    done
