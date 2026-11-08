#!/bin/sh
# Assemble the GitHub Pages site in a new directory (default _site): the pages in site/,
# with the web interface to the engine under engine/. Build the web interface first, with
# "npm run build:wasm" and "npm run build" in web/app.
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-_site}"

if [ -e "$out" ]; then
  echo "$out already exists" >&2
  exit 1
fi
if [ ! -f "$root/web/app/dist/index.html" ]; then
  echo "Build the web interface first (web/app/dist is missing)" >&2
  exit 1
fi

mkdir -p "$out"
find "$root/site" -maxdepth 1 -type f ! -name '*.sh' -exec cp {} "$out/" \;
cp -R "$root/web/app/dist" "$out/engine"
