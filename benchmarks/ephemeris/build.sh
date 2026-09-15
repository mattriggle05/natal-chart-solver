#!/bin/sh
set -eu
cd "$(dirname "$0")"
: "${ASTRONOMY_SOURCE:?Set ASTRONOMY_SOURCE to a checkout of cosinekitty/astronomy}"
expected=865d3da7d8112bbc7911238052c6af4aaf877181
actual=$(git -C "$ASTRONOMY_SOURCE" rev-parse HEAD)
[ "$actual" = "$expected" ] || { echo "Expected Astronomy Engine $expected, got $actual" >&2; exit 1; }
mkdir -p generated
cp "$ASTRONOMY_SOURCE/source/js/esm/astronomy.js" generated/astronomy.js
cp "$ASTRONOMY_SOURCE/LICENSE" generated/ASTRONOMY_LICENSE
wasm-pack build rust --target web --out-dir ../generated/baseline --release -- --locked --features baseline
wasm-pack build rust --target web --out-dir ../generated/astro --release -- --locked
emcc adapter.c -I "$ASTRONOMY_SOURCE/source/c" -O3 -sMODULARIZE=1 -sEXPORT_ES6=1 -sENVIRONMENT=worker -sFILESYSTEM=0 -sEXPORTED_FUNCTIONS='["_run","_angle","_representative_search","_search_value"]' -o generated/astronomy-c.mjs
node sizes.mjs > results/asset-sizes.json
