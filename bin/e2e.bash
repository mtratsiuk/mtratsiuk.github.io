#!/usr/bin/env bash

set -eu
set -o pipefail

UPDATE=false
CI=false

for arg in "$@"; do
  case "$arg" in
    -u|--update)
      UPDATE=true
      ;;
    --ci)
      CI=true
      ;;
  esac
done

cd "$(dirname "$0")"/..

RUSTACHE="./bin/rustache"
FAILED=0

for test_dir in ./e2e/*/; do
  test_name=$(basename "$test_dir")
  snapshot_file="${test_dir}${test_name}.snap"

  if [[ ! -f "${test_dir}index.mishon" || ! -f "${test_dir}index.mishtache" ]]; then
    echo "[e2e] FAIL $test_name — missing index.mishon or index.mishtache file"
    FAILED=1
    continue
  fi

  temp_output=$(mktemp)

  if ! "$RUSTACHE" --in="$test_dir" --out="$temp_output" 2>/dev/null; then
    echo "[e2e] FAIL $test_name — rustache failed to render"
    rm -f "$temp_output"
    FAILED=1
    continue
  fi

  if [[ "$UPDATE" == true ]]; then
    cp "$temp_output" "$snapshot_file"
    echo "[e2e] UPDATE $test_name"
  elif [[ -f "$snapshot_file" ]]; then
    if git diff --quiet --no-index "$temp_output" "$snapshot_file" 2>/dev/null; then
      echo "[e2e] PASS $test_name"
    else
      echo "[e2e] FAIL $test_name — snapshot mismatch"
      git diff --no-index --color "$snapshot_file" "$temp_output" || true
      FAILED=1
    fi
  elif [[ "$CI" == true ]]; then
    echo "[e2e] FAIL $test_name — snapshot missing (run without --ci to create)"
    FAILED=1
  else
    cp "$temp_output" "$snapshot_file"
    echo "[e2e] CREATE $test_name"
  fi

  rm -f "$temp_output"
done

if [[ "$FAILED" -eq 1 ]]; then
  exit 1
fi

echo "[e2e] All tests passed"
