#!/usr/bin/env bash
# Build the archive as static files in web/crates/rgapp/dist (see README.md).
#   web/build.sh                      production build (size-optimised, LTO, wasm-opt)
#   PUBLIC_URL=/repo/ web/build.sh    when the site is served from a sub-path
set -euo pipefail
cd "$(dirname "$0")"

python3 export.py                                       # the corpus files + manifest, from ../
cargo run --release -q -p rgcheck                       # load it as the browser will; every question must pass
# A build id for cache-busting the files Trunk doesn't hash (the worker, its wasm, the dataset).
export RG_BUILD="$(python3 -c 'import json; print(json.load(open("crates/rgapp/public/data/manifest.json"))["version"][:8])')"
(cd crates/rgapp && trunk build --release --cargo-profile dist ${PUBLIC_URL:+--public-url "$PUBLIC_URL"})
# The worker's wasm is loaded by rgworker_loader.js (no integrity hash), so it can be optimised after the build.
# Optional: a missing or too-old wasm-opt only costs size, never the build.
if command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --strip-debug --strip-producers --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-reference-types crates/rgapp/dist/rgworker_bg.wasm -o crates/rgapp/dist/rgworker_bg.opt.wasm \
    && mv crates/rgapp/dist/rgworker_bg.opt.wasm crates/rgapp/dist/rgworker_bg.wasm \
    || echo "wasm-opt failed; keeping the unoptimised worker"
fi
du -sh crates/rgapp/dist
ls -la crates/rgapp/dist | grep -E "wasm|js$"
