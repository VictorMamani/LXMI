#!/usr/bin/env python3
"""Own D3D11 fixture in a copied prefix; native, importer-only, full profile.

Never starts ZZZ, enters its live prefix or calls an injector. Requires an
explicit opt-in because it executes Proton and a hidden graphics window.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("zzz_experiment", HERE / "experiment.py")
experiment = importlib.util.module_from_spec(spec)
spec.loader.exec_module(experiment)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--hook-paths-only", action="store_true", help="Exercise original D3D11 exports after wrapper hooks, combined and separate device creation")
    parser.add_argument("--libraries-1-2-2", action="store_true", help="Reverify cached official Libraries 1.2.2 and substitute its DLLs in the private fixture only")
    args = parser.parse_args()
    experiment.require(os.environ.get("LXMI_CONFIRM_PRIVATE_GRAPHICS_DIAGNOSTIC") == "YES", "Explicit private-graphics diagnostic opt-in required")
    os.umask(0o077)
    session = experiment.STORE / "experiments/zzz-direct-15740e8101d0fd0eee6dffd9a5c0a138"
    plan = json.loads((session / "prepared.json").read_text())
    recovery = json.loads((session / "prefix-backup-manifest.json").read_text())
    experiment.require(experiment.inventory(session / "prefix-backup") == recovery, "Recovery copy changed")
    root = Path("/tmp") / ("lxmi-zzz-graphics-" + secrets.token_hex(16))
    root.mkdir(mode=0o700)
    print(json.dumps({"status": "preparing_private_graphics_diagnostic", "root": str(root)}), flush=True)
    binary = root / "lxmi-graphics-fixture.exe"
    subprocess.run(["x86_64-w64-mingw32-gcc", "-O2", "-Wall", "-Wextra", "-Werror", "-D_WIN32_WINNT=0x0601", "-DUNICODE", "-D_UNICODE", "-municode", str(HERE / "graphics_fixture.c"), "-o", str(binary), "-ldxguid"], check=True)
    compat = root / "compatdata"
    compat.mkdir(mode=0o700)
    shutil.copytree(session / "prefix-backup", compat / "pfx", symlinks=True)
    experiment.require(experiment.inventory(compat / "pfx") == recovery, "Private prefix clone mismatch")
    source = experiment.STORE / "profile-runtimes/zenless-zone-zero" / plan["runtime_id"] / "ZZMI"
    original_inventory = experiment.inventory(source)
    experiment.require(original_inventory == plan["original_inventory"], "Original selected runtime changed")
    libraries_review = experiment.review_libraries_1_2_2() if args.libraries_1_2_2 else None
    results = []
    cases = [("zzmi_hooked_with_profile", "hooked"), ("zzmi_device_hooked_with_profile", "hooked-device")] if args.hook_paths_only else [("native", "native"), ("zzmi_without_mods", "xxmi"), ("zzmi_with_profile", "xxmi")]
    for label, mode in cases:
        case = root / label
        case.mkdir(mode=0o700)
        shutil.copy2(binary, case / binary.name)
        if label != "native":
            def ignore(directory, names):
                return ["Mods"] if label == "zzmi_without_mods" and Path(directory) == source and "Mods" in names else []
            shutil.copytree(source, case / "ZZMI", symlinks=True, ignore=ignore)
            if libraries_review:
                expected = {item["relative_path"]: item for item in libraries_review["files"]}
                for name in ["d3d11.dll", "d3dcompiler_47.dll"]:
                    shutil.copy2(Path(libraries_review["payload_path"]) / name, case / "ZZMI" / name)
                    experiment.require(experiment.sha(case / "ZZMI" / name) == expected[name]["sha256"], "Updated diagnostic DLL hash mismatch")
            # Avoid Wine resolving the system d3d11 module by its special basename.
            # The fixture verifies this separate module's exact full path.
            shutil.copy2(case / "ZZMI/d3d11.dll", case / "ZZMI/lxmi-xxmi.dll")
            if label == "zzmi_without_mods":
                (case / "ZZMI/Mods").mkdir()
            ini = case / "ZZMI/d3dx.ini"
            content = ini.read_bytes()
            for pattern, value in [
                (rb"(?m)^target\s*=.*?(?=\r?$)", b"target = lxmi-graphics-fixture.exe"),
                (rb"(?m)^loader\s*=.*?(?=\r?$)", b"loader = lxmi-graphics-fixture.exe"),
                (rb"(?m)^log_level\s*=.*?(?=\r?$)", b"log_level = info"),
                (rb"(?m)^unbuffered\s*=.*?(?=\r?$)", b"unbuffered = 1"),
            ]:
                content, count = re.subn(pattern, value, content)
                experiment.require(count == 1, "Expected exactly one diagnostic INI setting")
            ini.write_bytes(content)
        variable = case / "runtime-variable"
        variable.mkdir(mode=0o700)
        allowed = ["HOME", "USER", "LOGNAME", "PATH", "DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY", "XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS", "LANG", "LC_ALL"]
        env = {key: os.environ[key] for key in allowed if key in os.environ}
        env.update(STEAM_COMPAT_DATA_PATH=str(compat), STEAM_COMPAT_CLIENT_INSTALL_PATH=str(experiment.STEAM), SteamAppId="0", SteamGameId="0", STEAM_COMPAT_APP_ID="0", PRESSURE_VESSEL_VARIABLE_DIR=str(variable), WINEDEBUG="+seh")
        argv = [str(experiment.RUNTIME / "_v2-entry-point"), "--verb=run", "--", str(experiment.PROTON / "proton"), "runinprefix", str(case / binary.name), mode]
        print(json.dumps({"status": "running_private_fixture", "case": label}), flush=True)
        with (case / "stdout.jsonl").open("xb") as stdout, (case / "stderr.log").open("xb") as stderr:
            process = subprocess.Popen(argv, cwd=case, env=env, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr)
            try:
                code = process.wait(timeout=60)
            except subprocess.TimeoutExpired:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
                code = None
        stages = []
        for line in (case / "stdout.jsonl").read_text(errors="replace").splitlines():
            try:
                stages.append(json.loads(line))
            except ValueError:
                pass
        log = case / "ZZMI/d3d11_log.txt"
        body = log.read_text(errors="replace") if log.is_file() else ""
        identity = any(stage.get("stage") == "xxmi_module_identity_verified" for stage in stages)
        completed = any(stage.get("stage") == "fixture_completed" for stage in stages)
        item = {"case": label, "mode": mode, "exit_code": code, "stages": stages, "log_present": log.is_file(), "log_bytes": log.stat().st_size if log.is_file() else 0, "wrapper_identity_verified": identity, "wrapper_initialization_complete": "D3D11 DLL successfully initialized" in body, "valid_wrapper_probe": label == "native" or (identity and log.is_file() and completed)}
        results.append(item)
        print(json.dumps(item), flush=True)
        # A timed-out Windows fixture could still be running: do not launch another.
        if code is None:
            break
    report = {"root": str(root), "source_session": session.name, "libraries_version": "1.2.2" if libraries_review else "1.1.7", "libraries_review": libraries_review, "cases": results, "real_game_started": False, "real_prefix_entered": False, "injector_called": False, "original_runtime_unchanged": experiment.inventory(source) == original_inventory, "recovery_copy_unchanged": experiment.inventory(session / "prefix-backup") == recovery}
    experiment.save(root / "report.json", report)
    print(json.dumps({"status": "diagnostic_finished", "report": str(root / "report.json")}), flush=True)


if __name__ == "__main__":
    main()
