#!/usr/bin/env bash

set -eu
set -o pipefail

cd "$(dirname "$0")"/..

CF_ZONE_ID="$CF_ZONE_ID"
CF_PURGE_TOKEN="$CF_PURGE_TOKEN"

curl -X POST "https://api.cloudflare.com/client/v4/zones/$CF_ZONE_ID/purge_cache" \
     -H "Authorization: Bearer $CF_PURGE_TOKEN" \
     -H "Content-Type: application/json" \
     --data '{"files":["https://misha.spris.dev/"]}' > /dev/null 2>&1
