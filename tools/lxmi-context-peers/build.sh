#!/usr/bin/env bash
set -euo pipefail
app_root="$(cd -- "$(dirname -- "$0")/../.." && pwd -P)"
out="$app_root/target/x86_64-pc-windows-gnu/release/lxmi-context-peers"
cc="${MINGW_CC:-x86_64-w64-mingw32-gcc}"
mkdir -p -- "$out"
for peer in a b; do
  define=0
  if [[ "$peer" == a ]]; then define=1; fi
  "$cc" -O2 -Wall -Wextra -Werror -municode -DPEER_A="$define" \
    "$app_root/tools/lxmi-context-peers/peer.c" -o "$out/lxmi-context-peer-$peer.exe"
done
file "$out"/*.exe
sha256sum "$out"/*.exe
