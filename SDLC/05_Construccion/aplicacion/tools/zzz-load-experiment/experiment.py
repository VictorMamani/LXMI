#!/usr/bin/env python3
"""Opt-in host-bound ZZZ experiment, separate from the desktop launch gate.

prepare: verify selected payload, compile helper, copy runtime and closed prefix.
run: wait for a manually started Steam game, verify its allowlisted context,
     consume this session once, then call official Inject in Steam Runtime 4
     (or, with --hook, official HookLibrary before ZZZ starts).
No historical consent, Steam config edits, game deployment, or automatic restore.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import stat
import subprocess
import sys
import time

STORE = Path.home() / ".local/share/lxmi"
STEAM = Path.home() / ".local/share/Steam"
COMPAT = Path("/media/university/Steam/SteamLibrary/steamapps/compatdata/4162040")
GAME = Path("/media/university/Steam/SteamLibrary/steamapps/common/Zenless Zone Zero/games/ZenlessZoneZero Game/ZenlessZoneZero.exe")
PROTON = STEAM / "steamapps/common/Proton - Experimental"
RUNTIME = STEAM / "steamapps/common/SteamLinuxRuntime_4"
HELPER = "LXMI ZZZ Loader.exe"
HERE = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def review_libraries_1_2_2():
    reviewer = HERE.parents[1] / "target/debug/examples/review_cached_libraries_1_2_2"
    require(reviewer.is_file(), "Build the cached-library signature reviewer first")
    review = json.loads(subprocess.check_output([str(reviewer)], timeout=30))
    expected_root = Path("/tmp/lxmi-wwmi-official-review/lxmi/packages/xxmi/711ddd3312acfdd420662f5ded293e2d6386b7fcd58136f00cf7a41226c00d77/payload")
    require(review.get("version") == "1.2.2" and review.get("payload_path") == str(expected_root), "Unexpected Libraries review")
    require(review.get("archive_signature") == "verified", "Libraries archive signature not verified")
    for name in ["3dmloader.dll", "d3d11.dll", "d3dcompiler_47.dll"]:
        require(review.get("component_signatures", {}).get(name) == "verified", f"Libraries signature missing: {name}")
    return review


def save(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def inventory(root):
    """Do not descend through symlinks, including dosdevices/z:."""
    require(root.is_dir() and not root.is_symlink(), f"Unsafe tree root: {root}")
    result = {}
    for current, dirs, names in os.walk(root, followlinks=False):
        for name in sorted(dirs + names):
            path = Path(current) / name
            info = path.lstat()
            item = {"mode": stat.S_IMODE(info.st_mode)}
            if stat.S_ISLNK(info.st_mode):
                item.update(kind="symlink", target=os.readlink(path))
            elif stat.S_ISREG(info.st_mode):
                item.update(kind="file", size=info.st_size, sha256=sha(path))
            elif stat.S_ISDIR(info.st_mode):
                item.update(kind="directory")
            else:
                raise RuntimeError(f"Special file in tree: {path}")
            result[str(path.relative_to(root))] = item
    return result


def game_context(executable=GAME, comm_prefix="ZenlessZone"):
    matches = []
    for proc in Path("/proc").iterdir():
        if not proc.name.isdigit():
            continue
        try:
            if proc.stat().st_uid != os.getuid():
                continue
            if not (proc / "comm").read_text().strip().startswith(comm_prefix):
                continue
            # Only inspect a named game candidate; retain allowlisted keys only.
            fields = (proc / "environ").read_bytes().split(b"\0")
            allowed = {"SteamAppId", "SteamGameId", "STEAM_COMPAT_DATA_PATH", "WINEPREFIX", "STEAM_COMPAT_TOOL_PATHS"}
            env = {}
            for field in fields:
                key, sep, value = field.partition(b"=")
                if sep and key.decode(errors="replace") in allowed:
                    env[key.decode()] = value.decode()
            args = (proc / "cmdline").read_bytes().split(b"\0")
            exact = False
            for argument in args:
                name = argument.decode(errors="replace").replace("\\", "/")
                if re.match(r"^[A-Za-z]:/", name):
                    drive = COMPAT / "pfx/dosdevices" / name[:2].lower()
                    candidate = drive / name[3:]
                elif name.startswith("/"):
                    candidate = Path(name)
                else:
                    continue
                if candidate.resolve() == executable.resolve():
                    exact = True
            matches.append({"pid": int(proc.name), "exact_image_argument": exact, "environment": env})
        except (OSError, UnicodeError):
            continue
    require(len(matches) <= 1, "Multiple ZZZ processes; stop and review")
    if not matches:
        return None
    match = matches[0]
    env = match["environment"]
    require(match["exact_image_argument"], "Game path does not match this host experiment")
    require(env.get("SteamAppId") == "4162040" or env.get("SteamGameId") == "4162040", "Steam AppID not verified")
    require(env.get("WINEPREFIX") and Path(env["WINEPREFIX"]).resolve() == (COMPAT / "pfx").resolve(), "Unexpected game prefix")
    require(env.get("STEAM_COMPAT_DATA_PATH") and Path(env["STEAM_COMPAT_DATA_PATH"]).resolve() == COMPAT.resolve(), "Unexpected compatdata")
    require(str(PROTON) in env.get("STEAM_COMPAT_TOOL_PATHS", "").split(":"), "Proton Experimental not observed in the game context")
    return match


def require_closed_prefix():
    require(game_context() is None, "Close ZZZ before copying its prefix")
    require(game_context(GAME.parents[2] / "HYP.exe", "HYP.exe") is None, "Close HoYoPlay before copying its prefix")
    for proc in Path("/proc").iterdir():
        if not proc.name.isdigit():
            continue
        try:
            if proc.stat().st_uid != os.getuid():
                continue
            if not (proc / "comm").read_text().strip().startswith("wineserver"):
                continue
            env = (proc / "environ").read_bytes().split(b"\0")
            for field in env:
                key, sep, value = field.partition(b"=")
                if sep and key == b"WINEPREFIX":
                    require(Path(os.fsdecode(value)).resolve() != (COMPAT / "pfx").resolve(), "The game wineserver is still active; wait for it to exit")
        except (OSError, UnicodeError):
            continue


def prepare(without_mods=False, libraries_1_2_2=False):
    require_closed_prefix()
    require(GAME.is_file(), "Game executable missing")
    require((COMPAT / "pfx").is_dir(), "Current external prefix missing")
    selection = json.loads((STORE / "mods/zenless-zone-zero/active-runtime.json").read_text())["current"]
    runtime_id = selection["runtime_id"]
    require(re.fullmatch(r"profile-[0-9a-f]{64}", runtime_id), "Invalid runtime ID")
    source = STORE / "profile-runtimes/zenless-zone-zero" / runtime_id
    manifest = json.loads((source / "lxmi-profile-runtime.json").read_text())
    require(manifest["game_id"] == "zenless-zone-zero" and manifest["runtime_id"] == runtime_id, "Manifest identity mismatch")
    source_tree = inventory(source / "ZZMI")
    expected = {f["relative_path"]: f for f in manifest["files"]}
    actual = {k: v for k, v in source_tree.items() if v["kind"] != "directory"}
    require(set(actual) == set(expected), "Runtime inventory differs from selected manifest")
    for name, item in actual.items():
        require(item["kind"] == "file" and item["size"] == expected[name]["size"] and item["sha256"] == expected[name]["sha256"], f"Altered runtime file: {name}")
    loader = STORE / manifest["base"]["loader_library_store_path"]
    require(sha(loader) == manifest["base"]["loader_library_sha256"], "Official loader hash mismatch")
    libraries_review = review_libraries_1_2_2() if libraries_1_2_2 else None
    session = STORE / "experiments" / ("zzz-direct-" + secrets.token_hex(16))
    session.mkdir(parents=True, mode=0o700)
    print(json.dumps({"status": "preparing", "session": str(session)}), flush=True)
    subprocess.run(["x86_64-w64-mingw32-gcc", "-O2", "-Wall", "-Wextra", "-Werror", "-D_WIN32_WINNT=0x0601", "-DUNICODE", "-D_UNICODE", "-municode", str(HERE / "loader.c"), "-o", str(session / HELPER)], check=True)
    def ignore_mods(directory, names):
        return ["Mods"] if without_mods and Path(directory) == source / "ZZMI" and "Mods" in names else []
    shutil.copytree(source / "ZZMI", session / "ZZMI", symlinks=True, ignore=ignore_mods)
    if without_mods:
        (session / "ZZMI/Mods").mkdir(mode=0o700)
    if libraries_review:
        updated = Path(libraries_review["payload_path"])
        expected_files = {item["relative_path"]: item for item in libraries_review["files"]}
        for name in ["d3d11.dll", "d3dcompiler_47.dll"]:
            shutil.copy2(updated / name, session / "ZZMI" / name)
            require(sha(session / "ZZMI" / name) == expected_files[name]["sha256"], "Updated library copy hash mismatch")
        loader = updated / "3dmloader.dll"
        save(session / "libraries-review.json", libraries_review)
    shutil.copy2(loader, session / "3dmloader.dll")
    if libraries_review:
        require(sha(session / "3dmloader.dll") == expected_files["3dmloader.dll"]["sha256"], "Updated loader copy hash mismatch")
    ini = session / "ZZMI/d3dx.ini"
    content = ini.read_bytes()
    content, count = re.subn(rb"(?m)^loader\s*=.*?(?=\r?$)", b"loader = LXMI ZZZ Loader.exe", content)
    require(count == 1, "Expected one loader setting")
    content, count = re.subn(rb"(?m)^log_level\s*=.*?(?=\r?$)", b"log_level = warning", content)
    require(count == 1, "Expected one logging setting")
    content, count = re.subn(rb"(?m)^unbuffered\s*=.*?(?=\r?$)", b"unbuffered = 1", content)
    require(count == 1, "Expected one unbuffered logging setting")
    ini.write_bytes(content)
    before = inventory(COMPAT / "pfx")
    shutil.copytree(COMPAT / "pfx", session / "prefix-backup", symlinks=True)
    require_closed_prefix()
    require(inventory(COMPAT / "pfx") == before, "Prefix changed during backup")
    require(inventory(session / "prefix-backup") == before, "Backup verification failed")
    save(session / "prefix-backup-manifest.json", before)
    metadata = {}
    for name in ["version", "config_info", "tracked_files"]:
        path = COMPAT / name
        if path.is_file() and not path.is_symlink():
            shutil.copy2(path, session / ("compatdata-" + name))
            metadata[name] = sha(path)
    prepared = inventory(session / "ZZMI")
    bindings = {str(p): sha(p) for p in [GAME, GAME.parents[2] / "HYP.exe", PROTON / "proton", PROTON / "version", RUNTIME / "_v2-entry-point", RUNTIME / "VERSIONS.txt", session / HELPER, session / "3dmloader.dll", HERE / "experiment.py", HERE / "loader.c", source / "lxmi-profile-runtime.json", STORE / "mods/zenless-zone-zero/active-runtime.json"]}
    if libraries_review:
        for path in [session / "libraries-review.json", HERE.parents[1] / "target/debug/examples/review_cached_libraries_1_2_2"]:
            bindings[str(path)] = sha(path)
    profile = {"name": "ZZMI — diagnóstico sin mods", "mod_ids": []} if without_mods else manifest["profile"]
    save(session / "prepared.json", {"schema": 1, "created": time.time(), "session": str(session), "prefix": str(COMPAT / "pfx"), "prefix_identity": [(COMPAT / "pfx").stat().st_dev, (COMPAT / "pfx").stat().st_ino], "runtime_id": runtime_id, "profile": profile, "source_profile": manifest["profile"], "without_mods": without_mods, "libraries_version": "1.2.2" if libraries_review else manifest["base"]["libraries_version"], "libraries_review": libraries_review, "proton_version": (PROTON / "version").read_text().strip(), "bindings": bindings, "prepared_inventory": prepared, "original_inventory": source_tree, "compatdata_metadata": metadata, "backup_content_verified": True, "real_restore_tested": False, "loader_executed": False})
    print(json.dumps({"status": "prepared", "session": str(session), "mods": len(profile["mod_ids"]), "backup_entries": len(before), "backup_bytes": sum(x.get("size", 0) for x in before.values())}), flush=True)


def run(session, wait_seconds, arm_before_game=False, hook=False):
    require(not hook or arm_before_game, "--hook requires --arm-before-game: the hook must exist before ZZZ starts")
    require(os.environ.get("LXMI_CONFIRM_ZZZ_MOD_EXPERIMENT") == "YES", "Explicit opt-in LXMI_CONFIRM_ZZZ_MOD_EXPERIMENT=YES required")
    require(session.parent == STORE / "experiments" and re.fullmatch(r"zzz-direct-[0-9a-f]{32}", session.name) and not session.is_symlink(), "Invalid session")
    plan = json.loads((session / "prepared.json").read_text())
    require(time.time() - plan["created"] < 86400, "Prepared session expired")
    require(not (session / "consumed.json").exists(), "Session already consumed; inspect its result")
    for path, digest in plan["bindings"].items():
        require(sha(Path(path)) == digest, f"Prepared binding changed: {path}")
    require(inventory(session / "ZZMI") == plan["prepared_inventory"], "Working runtime changed")
    backup = json.loads((session / "prefix-backup-manifest.json").read_text())
    require(inventory(session / "prefix-backup") == backup, "Recovery copy changed")
    require([(COMPAT / "pfx").stat().st_dev, (COMPAT / "pfx").stat().st_ino] == plan["prefix_identity"], "Prefix identity changed")
    context = None
    if arm_before_game:
        require(game_context() is None, "Keep ZZZ closed while arming the helper")
        launcher = game_context(GAME.parents[2] / "HYP.exe", "HYP.exe")
        require(launcher is not None, "Open HoYoPlay through Steam first, then leave its Play button untouched")
        save(session / "observed-launcher-context.json", launcher)
    else:
        print('{"status":"waiting_for_manual_steam_launch"}', flush=True)
        deadline = time.monotonic() + wait_seconds
        while time.monotonic() < deadline:
            context = game_context()
            if context:
                break
            time.sleep(1)
        require(context is not None, "No game observed; no loader executed")
    # Capture files before entry; natural changes due to Steam/game remain separate.
    save(session / "prefix-before-entry.json", inventory(COMPAT / "pfx"))
    if not arm_before_game:
        context = game_context()
        require(context is not None, "Game exited before entry")
        save(session / "observed-context.json", context)
    # O_EXCL journal: failure, interruption and timeout all consume this session.
    operation = "one_official_hook" if hook else "one_early_official_inject" if arm_before_game else "one_official_inject"
    save(session / "consumed.json", {"time": time.time(), "operation": operation, "historical_0_8_8_consent_used": False})
    variable = session / "runtime-variable"
    variable.mkdir(mode=0o700)
    env_keys = ["HOME", "USER", "LOGNAME", "PATH", "DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY", "XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS", "LANG", "LC_ALL"]
    env = {k: os.environ[k] for k in env_keys if k in os.environ}
    env.update(STEAM_COMPAT_DATA_PATH=str(COMPAT), STEAM_COMPAT_CLIENT_INSTALL_PATH=str(STEAM), SteamAppId="4162040", SteamGameId="4162040", STEAM_COMPAT_APP_ID="4162040", PRESSURE_VESSEL_VARIABLE_DIR=str(variable), WINEDEBUG="-all")
    if hook:
        # runinprefix skips Proton's setup_prefix, which is where the DXVK native
        # overrides come from; without them HookLibrary resolves ZZMI\\d3d11.dll as
        # Wine's builtin d3d11 and fails with 300 (hook isolation fixture).
        env["WINEDLLOVERRIDES"] = "d3d11,d3d10core,d3d9,dxgi,d3d12,d3d12core=n"
    mode = "hook-wait" if hook else "wait-inject" if arm_before_game else "inject"
    argv = [str(RUNTIME / "_v2-entry-point"), "--verb=run", "--", str(PROTON / "proton"), "runinprefix", str(session / HELPER), mode]
    print(json.dumps({"status": "executing_" + operation}), flush=True)
    outcome = {"status": "interrupted", "mods_visible": "pending_user", "session": str(session)}
    try:
        with (session / "helper.stdout.jsonl").open("xb") as stdout, (session / "helper.stderr.log").open("xb") as stderr:
            process = subprocess.Popen(argv, env=env, cwd=session, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr)
            try:
                code = process.wait(timeout=360 if arm_before_game else 60)
            except subprocess.TimeoutExpired:
                # Terminate only our launcher. Never signal a shared wineserver/game.
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
                raise RuntimeError("Helper launcher timed out; session consumed, inspect logs before further work")
        lines = (session / "helper.stdout.jsonl").read_text(errors="replace").splitlines()
        reports = []
        for line in lines:
            try:
                reports.append(json.loads(line))
            except json.JSONDecodeError:
                pass
        loaded = any((r.get("status") == "inject_returned" and r.get("inject_code") == 0 or r.get("status") == "hook_target_observed") and r.get("exact_module_present") == 1 for r in reports)
        called = any(r.get("status") in ("calling_official_inject", "hook_library_returned") for r in reports)
        alive = any(r.get("status") == "original_game_alive_after_load" for r in reports)
        exited = any(r.get("status") == "original_game_exited_after_load" for r in reports)
        if loaded and exited:
            state = "module_loaded_original_game_exited"
        elif loaded and alive and code == 0:
            state = "module_loaded_health_window_passed_visual_pending"
        else:
            state = "failed" if called else "no_inject_attempted"
        outcome.update(status=state, exit_code=code, reports=reports, loader_mode="hook" if hook else "inject", module_loaded=loaded, inject_called=called and not hook, hook_installed=any(r.get("status") == "hook_library_returned" and r.get("code") == 0 for r in reports), hook_removed=any(r.get("status") == "unhook_library_returned" and r.get("code") == 0 for r in reports), original_game_alive_after_15_seconds=alive)
        if exited:
            outcome["mods_visible"] = "not_verified_original_game_exited"
        if arm_before_game:
            context = None if exited else game_context()
            save(session / "observed-context.json", context)
            outcome["steam_game_context_verified_after_entry"] = context is not None
    except BaseException as error:
        outcome.update(status="failed_or_interrupted", error=str(error))
        raise
    finally:
        save(session / "result.json", outcome)
        save(session / "prefix-after-entry.json", inventory(COMPAT / "pfx"))
        print(json.dumps(outcome, ensure_ascii=False), flush=True)
    require(outcome["status"] == "module_loaded_health_window_passed_visual_pending", "Load did not pass the original game's 15-second health observation; inspect evidence")


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["prepare", "run"])
    parser.add_argument("--session", type=Path)
    parser.add_argument("--wait-seconds", type=int, default=240)
    parser.add_argument("--arm-before-game", action="store_true", help="Arm a fixed Windows helper first; accepts only a new ZZZ process, waits at most 300 seconds")
    parser.add_argument("--hook", action="store_true", help="run only, with --arm-before-game: official HookLibrary before ZZZ starts instead of Inject; unhooked as soon as ZZMI is seen in the new ZZZ or on timeout")
    parser.add_argument("--without-mods", action="store_true", help="prepare only: omit Mods from this working copy for an importer-only diagnostic; keep saved profile unchanged")
    parser.add_argument("--libraries-1-2-2", action="store_true", help="prepare only: reverify cached official Libraries 1.2.2 signatures and use its DLLs only in this session")
    args = parser.parse_args()
    require(1 <= args.wait_seconds <= 600, "Wait must be 1..600 seconds")
    if args.operation == "prepare":
        require(not args.hook, "--hook belongs to run")
        prepare(args.without_mods, args.libraries_1_2_2)
    else:
        require(not args.without_mods, "--without-mods belongs to prepare; run uses its bound plan")
        require(not args.libraries_1_2_2, "--libraries-1-2-2 belongs to prepare; run uses its bound plan")
        require(args.session is not None, "--session required")
        run(args.session, args.wait_seconds, args.arm_before_game, args.hook)


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, subprocess.SubprocessError) as error:
        print(json.dumps({"status": "stopped", "error": str(error)}, ensure_ascii=False), file=sys.stderr)
        sys.exit(1)
