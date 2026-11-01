#!/usr/bin/env bash
# Builds the phone's protocol module into apps/iphone/ (rovi_protocol.js, rovi_protocol_bg.wasm;
# generated, not committed). Needs rustup's stable toolchain with the wasm32-unknown-unknown target
# and wasm-bindgen 0.2.121, both from the project nix-shell or CI.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out="$here/../../apps/iphone"
if command -v rustup >/dev/null; then cargo=(rustup run stable cargo); else cargo=(cargo); fi
"${cargo[@]}" build --locked --release --target wasm32-unknown-unknown --manifest-path "$here/Cargo.toml" >&2
wasm-bindgen --target web --no-typescript --out-name rovi_protocol --out-dir "$out" \
  "$here/target/wasm32-unknown-unknown/release/rovi_protocol.wasm"
