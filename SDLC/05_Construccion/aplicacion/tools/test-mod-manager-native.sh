#!/usr/bin/env bash
# Opt-in real Tauri test. Start `npm run dev` separately; no games or OS input.
set -euo pipefail
cd "$(dirname "$0")/.."
python3 -c 'import urllib.request; urllib.request.urlopen("http://127.0.0.1:1420", timeout=3).close()' || {
  echo 'Start apps/lxmi-desktop/npm run dev first (port 1420).' >&2
  exit 1
}
LXMI_NATIVE_MOD_SMOKE_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/lxmi-mod-native-smoke-XXXXXX")
export LXMI_NATIVE_MOD_SMOKE_ROOT
export XDG_DATA_HOME="$LXMI_NATIVE_MOD_SMOKE_ROOT/data"
export XDG_CACHE_HOME="$LXMI_NATIVE_MOD_SMOKE_ROOT/cache"
mkdir -p "$XDG_DATA_HOME" "$XDG_CACHE_HOME"
if [[ -n "${LXMI_NATIVE_MOD_SMOKE_SEED:-}" ]]; then
python3 - <<'PY'
import os, shutil, stat
from pathlib import Path
source=Path(os.environ['LXMI_NATIVE_MOD_SMOKE_SEED'])
target=Path(os.environ['XDG_DATA_HOME'])/'lxmi'
target.mkdir(mode=0o700)
for name in ('packages',):
    tree=source/name
    for parent,dirs,files in os.walk(tree,followlinks=False):
        for leaf in dirs+files:
            mode=(Path(parent)/leaf).lstat().st_mode
            assert stat.S_ISDIR(mode) or stat.S_ISREG(mode), 'No links or special files in seed'
    shutil.copytree(tree,target/name)
PY
fi
echo "Isolated native smoke root: $LXMI_NATIVE_MOD_SMOKE_ROOT"
LXMI_NATIVE_MOD_SMOKE_PHASE=exercise cargo run --offline -p lxmi-desktop --example mod_manager_native_smoke 2>&1 | tee "$LXMI_NATIVE_MOD_SMOKE_ROOT/exercise.log"
python3 - <<'PY'
import os
from pathlib import Path
from zipfile import ZipFile
root = Path(os.environ['LXMI_NATIVE_MOD_SMOKE_ROOT'])
with ZipFile(root / 'source-c.zip', 'w') as archive:
    archive.writestr('skin/skin.ini', '; synthetic LXMI ZIP fixture\n[Constants]\nglobal $lxmi_test = 3\n')
PY
LXMI_NATIVE_MOD_SMOKE_PHASE=verify cargo run --offline -p lxmi-desktop --example mod_manager_native_smoke 2>&1 | tee "$LXMI_NATIVE_MOD_SMOKE_ROOT/verify.log"
if [[ -n "${LXMI_NATIVE_MOD_SMOKE_SEED:-}" ]]; then
  LXMI_NATIVE_MOD_SMOKE_PHASE=activation cargo run --offline -p lxmi-desktop --example mod_manager_native_smoke 2>&1 | tee "$LXMI_NATIVE_MOD_SMOKE_ROOT/activation.log"
fi
python3 - <<'PY'
import hashlib
import json
import os
from pathlib import Path
root = Path(os.environ['LXMI_NATIVE_MOD_SMOKE_ROOT'])
for phase in ('exercise', 'verify'):
    report = json.loads((root / f'native-{phase}.json').read_text())
    assert report['status'] == 'passed', report
base = root / 'data/lxmi/mods'
for game in ('wuthering-waves', 'zenless-zone-zero'):
    for package in (base / game / 'packages').iterdir():
        manifest = json.loads((package / 'lxmi-mod.json').read_text())
        for item in manifest['files']:
            content = (package / 'payload' / item['relative_path']).read_bytes()
            assert len(content) == item['size']
            assert hashlib.sha256(content).hexdigest() == item['sha256']
prepared = list((base / 'wuthering-waves/prepared').iterdir())
assert len(prepared) == 1, 'Identical preparation after restart must reuse the snapshot'
manifest = json.loads((prepared[0] / 'lxmi-profile.json').read_text())
assert len(manifest['profile']['mod_ids']) == 2
for item in manifest['files']:
    content = (prepared[0] / 'Mods' / item['relative_path']).read_bytes()
    assert hashlib.sha256(content).hexdigest() == item['sha256']
assert (root / 'source-a/skin.ini').read_bytes() == b'; synthetic LXMI fixture A\r\n[Constants]\r\nglobal $lxmi_test = 1\r\n'
assert (root / 'source-b/skin.ini').read_bytes() == b'; synthetic LXMI fixture B\n[Constants]\nglobal $lxmi_test = 2\n'
report = {'status': 'passed', 'importedPayloadHashes': 'PASS', 'preparedSnapshotHashes': 'PASS', 'snapshotReusedAfterRestart': True, 'originalBytesUnchanged': True, 'wutheringWavesMods': len(list((base / 'wuthering-waves/packages').iterdir())), 'zenlessZoneZeroMods': len(list((base / 'zenless-zone-zero/packages').iterdir()))}
(root / 'native-filesystem.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))
PY
echo "Evidence preserved at $LXMI_NATIVE_MOD_SMOKE_ROOT"
