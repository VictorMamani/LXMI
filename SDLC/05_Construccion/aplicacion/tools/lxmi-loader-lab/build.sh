#!/usr/bin/env bash
set -euo pipefail

app_root="$(cd -- "$(dirname -- "$0")/../.." && pwd -P)"
out="$app_root/target/x86_64-pc-windows-gnu/release/lxmi-loader-lab"
cc="${MINGW_CC:-x86_64-w64-mingw32-gcc}"

if ! command -v "$cc" >/dev/null 2>&1; then
  printf 'No se encontró el compilador MinGW: %s\n' "$cc" >&2
  exit 1
fi

mkdir -p -- "$out"
common=(-O2 -Wall -Wextra -Werror -D_WIN32_WINNT=0x0601 -DUNICODE -D_UNICODE)

"$cc" "${common[@]}" -municode \
  "$app_root/tools/lxmi-loader-lab/target.c" \
  -o "$out/lxmi-loader-test-target.exe"

"$cc" "${common[@]}" -municode \
  "$app_root/tools/lxmi-loader-lab/runner.c" \
  -o "$out/lxmi-loader-test-runner.exe"

"$cc" "${common[@]}" -shared \
  "$app_root/tools/lxmi-loader-lab/test_dll.c" \
  -o "$out/lxmi-loader-test.dll"

file "$out/lxmi-loader-test-target.exe" "$out/lxmi-loader-test-runner.exe" "$out/lxmi-loader-test.dll"
sha256sum "$out/lxmi-loader-test-target.exe" "$out/lxmi-loader-test-runner.exe" "$out/lxmi-loader-test.dll"
