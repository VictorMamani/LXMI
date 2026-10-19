#!/usr/bin/env bash
set -euo pipefail

app_root="$(cd -- "$(dirname -- "$0")/.." && pwd -P)"
target="x86_64-pc-windows-gnu"
binary="$app_root/target/$target/release/lxmi-bridge-helper.exe"

if [[ ! -f "$binary" || -L "$binary" ]]; then
  printf 'Helper not found. Build it for %s first:\n' "$target" >&2
  printf '  cargo build --manifest-path tools/lxmi-bridge-helper/Cargo.toml --target %s --release\n' "$target" >&2
  exit 1
fi

if [[ -v XDG_DATA_HOME ]]; then
  data_home="$XDG_DATA_HOME"
else
  data_home="$HOME/.local/share"
fi
if [[ "$data_home" != /* ]]; then
  printf 'XDG_DATA_HOME must be an absolute path.\n' >&2
  exit 1
fi
managed_root="$data_home/lxmi"
helpers_dir="$managed_root/helpers"

if [[ -L "$managed_root" || -L "$helpers_dir" ]]; then
  printf 'A symbolic link in the LXMI managed directory was rejected.\n' >&2
  exit 1
fi
mkdir -p -- "$helpers_dir"
if [[ -L "$managed_root" || -L "$helpers_dir" ]]; then
  printf 'The helper destination changed during preparation.\n' >&2
  exit 1
fi

binary_tmp="$(mktemp "$helpers_dir/.lxmi-bridge-helper.XXXXXX")"
manifest_tmp="$(mktemp "$helpers_dir/.lxmi-bridge-manifest.XXXXXX")"
cleanup() {
  rm -f -- "$binary_tmp" "$manifest_tmp"
}
trap cleanup EXIT

cp -- "$binary" "$binary_tmp"
sha256="$(sha256sum "$binary_tmp" | cut -d ' ' -f 1)"
printf '{\n  "protocol_version": 1,\n  "helper_version": "0.6.0",\n  "build_target": "%s",\n  "sha256": "%s"\n}\n' "$target" "$sha256" >"$manifest_tmp"
chmod 0644 "$binary_tmp" "$manifest_tmp"
mv -f -- "$binary_tmp" "$helpers_dir/lxmi-bridge-helper.exe"
mv -f -- "$manifest_tmp" "$helpers_dir/lxmi-bridge-helper.json"
trap - EXIT

printf 'Helper installed in LXMI storage:\n  %s\nSHA-256: %s\n' \
  "$helpers_dir/lxmi-bridge-helper.exe" "$sha256"
