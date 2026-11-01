#!/usr/bin/env bash
# Prints a directory holding an importable `rovi_protocol` built as a portable (manylinux, abi3)
# wheel, for Python outside nix-shell, e.g. the ROS 2 Docker containers: mount it and put it on
# PYTHONPATH. Rebuilds when the binding or crate sources change. Needs cargo and maturin (taken
# from `nix-shell -p maturin` when not on PATH).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out="$here/target/pymodule"
stamp=$(cat "$here/Cargo.toml" "$here/Cargo.lock" "$here"/src/*.rs "$here"/../../crates/rovi-protocol/Cargo.toml \
  "$here"/../../crates/rovi-protocol/src/*.rs | sha256sum | cut -c1-16)
if [ ! -f "$out/.stamp" ] || [ "$(cat "$out/.stamp")" != "$stamp" ]; then
  wheels="$here/target/wheels-module"
  rm -rf "$wheels" "$out"
  build=(maturin build --locked --release -m "$here/Cargo.toml" -o "$wheels")
  if command -v maturin >/dev/null; then "${build[@]}" >&2; else nix-shell -p maturin --run "${build[*]}" >&2; fi
  mkdir -p "$out"
  python3 -m zipfile -e "$wheels"/rovi_protocol-*.whl "$out"
  echo "$stamp" > "$out/.stamp"
fi
echo "$out"
