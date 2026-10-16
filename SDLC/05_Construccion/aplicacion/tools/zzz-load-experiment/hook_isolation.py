#!/usr/bin/env python3
"""Hook-mode isolation fixture: official HookLibrary in LXMI-owned prefixes.

Never starts ZZZ/HoYoPlay/Steam, enters the game prefix or calls Inject. Uses
two new Proton prefixes under LXMI storage: A holds the controller's global
WH_CBT hook; B runs the same target image while A's hook is installed.
Requires an explicit opt-in because it executes Proton and hidden windows.
"""
import importlib.util
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import subprocess
import time

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("zzz_experiment", HERE / "experiment.py")
experiment = importlib.util.module_from_spec(spec)
spec.loader.exec_module(experiment)
require = experiment.require
IMAGES = {"controller": "lxmi-hook-loader.exe", "target": "lxmi-hook-target.exe", "bystander": "lxmi-hook-bystander.exe"}


def selected_runtime():
    """Same identity checks as experiment.prepare(), without touching the game prefix."""
    selection = json.loads((experiment.STORE / "mods/zenless-zone-zero/active-runtime.json").read_text())["current"]
    runtime_id = selection["runtime_id"]
    require(re.fullmatch(r"profile-[0-9a-f]{64}", runtime_id), "Invalid runtime ID")
    source = experiment.STORE / "profile-runtimes/zenless-zone-zero" / runtime_id
    manifest = json.loads((source / "lxmi-profile-runtime.json").read_text())
    require(manifest["game_id"] == "zenless-zone-zero" and manifest["runtime_id"] == runtime_id, "Manifest identity mismatch")
    tree = experiment.inventory(source / "ZZMI")
    expected = {f["relative_path"]: f for f in manifest["files"]}
    actual = {k: v for k, v in tree.items() if v["kind"] != "directory"}
    require(set(actual) == set(expected), "Runtime inventory differs from selected manifest")
    for name, item in actual.items():
        require(item["kind"] == "file" and item["sha256"] == expected[name]["sha256"], f"Altered runtime file: {name}")
    loader = experiment.STORE / manifest["base"]["loader_library_store_path"]
    require(experiment.sha(loader) == manifest["base"]["loader_library_sha256"], "Official loader hash mismatch")
    return runtime_id, source, tree, loader, manifest["base"]["loader_library_sha256"]


def environment(compat, variable):
    allowed = ["HOME", "USER", "LOGNAME", "PATH", "DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY", "XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS", "LANG", "LC_ALL"]
    env = {key: os.environ[key] for key in allowed if key in os.environ}
    # +pid prefixes each trace line with the Windows PID; +loaddll records every
    # mapping attempt of the XXMI module, including rejected ones in other tasks.
    env.update(STEAM_COMPAT_DATA_PATH=str(compat), STEAM_COMPAT_CLIENT_INSTALL_PATH=str(experiment.STEAM), SteamAppId="0", SteamGameId="0", STEAM_COMPAT_APP_ID="0", PRESSURE_VESSEL_VARIABLE_DIR=str(variable), WINEDEBUG="+pid,+loaddll,+seh")
    # Proton adds its DXVK/vkd3d-proton native overrides only in setup_prefix,
    # which `runinprefix` skips; without them Wine resolves ZZMI\\d3d11.dll as its
    # builtin d3d11 (no CBTProc). Match what a Steam-started game receives.
    env["WINEDLLOVERRIDES"] = "d3d11,d3d10core,d3d9,dxgi,d3d12,d3d12core=n"
    return env


def proton(root, compat, label, image, role, verb="runinprefix"):
    variable = root / f"runtime-variable-{label}"
    variable.mkdir(mode=0o700)
    # `run` only initialises these fresh LXMI-owned prefixes (it does not relay
    # the fixture's stdout); measured cases use `runinprefix`.
    argv = [str(experiment.RUNTIME / "_v2-entry-point"), "--verb=run", "--", str(experiment.PROTON / "proton"), verb, str(root / "bin" / image), role]
    stdout = (root / f"{label}.stdout.jsonl").open("xb")
    stderr = (root / f"{label}.stderr.log").open("xb")
    process = subprocess.Popen(argv, cwd=root / "bin", env=environment(compat, variable), stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr)
    return process, stdout, stderr


def finish(handle, timeout):
    process, stdout, stderr = handle
    try:
        code = process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        code = None
    stdout.close()
    stderr.close()
    return code


def stages(path):
    result = []
    for line in path.read_text(errors="replace").splitlines():
        try:
            result.append(json.loads(line))
        except ValueError:
            pass
    return result


def wait_stage(path, stage, process, timeout):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if any(item.get("stage") == stage for item in stages(path)):
            return True
        if process.poll() is not None:
            return False
        time.sleep(0.5)
    return False


def xxmi_mappings(log, xxmi_windows_path):
    """PIDs whose loader traced the exact XXMI path, split into loads/unloads."""
    needle = xxmi_windows_path.replace("\\", "\\\\").lower()
    loaded, unloaded = {}, {}
    for line in log.read_text(errors="replace").splitlines():
        if needle not in line.lower():
            continue
        match = re.match(r"^([0-9a-f]{4,8}):", line)
        pid = int(match.group(1), 16) if match else -1
        if "Loaded L" in line:
            loaded[pid] = loaded.get(pid, 0) + 1
        elif "Unloaded module" in line:
            unloaded[pid] = unloaded.get(pid, 0) + 1
    return {"loaded_by_pid": loaded, "unloaded_by_pid": unloaded}


def first(items, role, stage):
    for item in items:
        if item.get("role") == role and item.get("stage") == stage:
            return item.get("value", item.get("module"))
    return None


def main():
    require(os.environ.get("LXMI_CONFIRM_HOOK_ISOLATION_FIXTURE") == "YES", "Explicit hook-isolation fixture opt-in required")
    os.umask(0o077)
    runtime_id, source, original_tree, loader, loader_sha = selected_runtime()
    root = experiment.STORE / "tests" / ("hook-isolation-" + secrets.token_hex(16))
    root.mkdir(mode=0o700, parents=False)
    print(json.dumps({"status": "preparing_hook_isolation_fixture", "root": str(root)}), flush=True)
    (root / "bin").mkdir(mode=0o700)
    (root / "logs").mkdir(mode=0o700)
    built = root / "hook_fixture.exe"
    subprocess.run(["x86_64-w64-mingw32-gcc", "-O2", "-Wall", "-Wextra", "-Werror", "-D_WIN32_WINNT=0x0601", "-DUNICODE", "-D_UNICODE", "-municode", str(HERE / "hook_fixture.c"), "-o", str(built), "-ldxguid", "-lpsapi"], check=True)
    for image in IMAGES.values():
        shutil.copy2(built, root / "bin" / image)
    shutil.copy2(loader, root / "3dmloader.dll")
    require(experiment.sha(root / "3dmloader.dll") == loader_sha, "Loader copy hash mismatch")
    # Empty Mods: the question is loader scope, not mod content.
    shutil.copytree(source / "ZZMI", root / "ZZMI", symlinks=True, ignore=lambda d, names: ["Mods"] if Path(d) == source / "ZZMI" else [])
    (root / "ZZMI/Mods").mkdir()
    ini = root / "ZZMI/d3dx.ini"
    content = ini.read_bytes()
    for pattern, value in [
        (rb"(?m)^target\s*=.*?(?=\r?$)", b"target = " + IMAGES["target"].encode()),
        (rb"(?m)^loader\s*=.*?(?=\r?$)", b"loader = " + IMAGES["controller"].encode()),
        (rb"(?m)^log_level\s*=.*?(?=\r?$)", b"log_level = info"),
        (rb"(?m)^unbuffered\s*=.*?(?=\r?$)", b"unbuffered = 1"),
    ]:
        content, count = re.subn(pattern, value, content)
        require(count == 1, "Expected exactly one fixture INI setting")
    ini.write_bytes(content)
    hashes = {name: experiment.sha(path) for name, path in [("hook_fixture.exe", built), ("hook_fixture.c", HERE / "hook_fixture.c"), ("hook_isolation.py", Path(__file__).resolve()), ("3dmloader.dll", root / "3dmloader.dll"), ("d3d11.dll", root / "ZZMI/d3d11.dll"), ("proton", experiment.PROTON / "proton")]}
    compat_a, compat_b = root / "compat-a", root / "compat-b"
    compat_a.mkdir(mode=0o700)
    compat_b.mkdir(mode=0o700)
    # Windows path of the XXMI module as both prefixes see it through z: -> /.
    xxmi_windows = "Z:" + str(root / "ZZMI/d3d11.dll").replace("/", "\\")
    timeline = []

    for label, compat in [("a-init", compat_a), ("b-init", compat_b)]:
        print(json.dumps({"status": "initialising_prefix", "case": label}), flush=True)
        code = finish(proton(root, compat, label, IMAGES["bystander"], "bystander", verb="run"), 300)
        timeline.append({"case": label, "exit_code": code})
        require(code is not None, "Prefix initialisation timed out; stop and inspect")
    print(json.dumps({"status": "baseline_target_prefix_b"}), flush=True)
    code = finish(proton(root, compat_b, "b-baseline", IMAGES["target"], "target-early"), 240)
    timeline.append({"case": "b_baseline_no_hook", "exit_code": code})
    require(code is not None, "Baseline fixture timed out; stop and inspect")

    print(json.dumps({"status": "installing_hook_prefix_a"}), flush=True)
    controller = proton(root, compat_a, "a-controller", IMAGES["controller"], "controller")
    installed = wait_stage(root / "a-controller.stdout.jsonl", "hook_installed", controller[0], 240)
    if installed:
        print(json.dumps({"status": "cross_prefix_target_while_hook_active"}), flush=True)
        code = finish(proton(root, compat_b, "b-while-hooked", IMAGES["target"], "target-early"), 120)
        timeline.append({"case": "b_target_while_a_hooked", "exit_code": code})
    # Always release the controller: on failure it unhooks after the go file.
    (root / "go").touch()
    code = finish(controller, 300)
    timeline.append({"case": "a_controller", "exit_code": code})

    a_items = stages(root / "a-controller.stdout.jsonl")
    pids = {}
    for item in a_items:
        pids.setdefault(item.get("pid"), set()).add(item.get("role"))
    a_maps = xxmi_mappings(root / "a-controller.stderr.log", xxmi_windows)
    b_maps = {label: xxmi_mappings(root / f"{label}.stderr.log", xxmi_windows) for label in ["b-baseline", "b-while-hooked"] if (root / f"{label}.stderr.log").exists()}
    other_a_pids = sorted(pid for pid in a_maps["loaded_by_pid"] if pid not in pids)
    log = root / "ZZMI/d3d11_log.txt"
    body = log.read_text(errors="replace") if log.is_file() else ""
    roles = {}
    for item in a_items:
        if item.get("stage") == "started":
            roles.setdefault(item["role"], []).append(item["pid"])
    target_runs = [pid for pid in roles.get("target-early", []) + roles.get("target-late", [])]

    def by_pid(pid, stage):
        for item in a_items:
            if item.get("pid") == pid and item.get("stage") == stage:
                return item.get("value", item.get("module"))
        return None

    target_cases = [{
        "pid": pid,
        "role": next(r for r in ("target-early", "target-late") if pid in roles.get(r, [])),
        "mapped_at_start": by_pid(pid, "xxmi_mapped_at_start"),
        "mapped_after_window": by_pid(pid, "xxmi_mapped_after_window"),
        "mapped_before_device": by_pid(pid, "xxmi_mapped_before_device"),
        "create_device_hresult": by_pid(pid, "create_device_hresult"),
        "device_vtable_module": by_pid(pid, "device_vtable_module"),
        "device_create_buffer_module": by_pid(pid, "device_create_buffer_module"),
        "present_hresult": by_pid(pid, "present_hresult"),
        "completed": by_pid(pid, "target_completed") is not None,
    } for pid in target_runs]
    b_items = {label: stages(root / f"{label}.stdout.jsonl") for label in ["b-baseline", "b-while-hooked"] if (root / f"{label}.stdout.jsonl").exists()}
    report = {
        "recorded_local_time": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "scope": "Official 3dmloader HookLibrary (global WH_CBT) scope under Proton, LXMI-owned prefixes and images only",
        "root": str(root),
        "runtime_id": runtime_id,
        "hashes": hashes,
        "proton": (experiment.PROTON / "version").read_text().strip() if (experiment.PROTON / "version").is_file() else None,
        "timeline": timeline,
        "hook_library_returned": first(a_items, "controller", "hook_library_returned"),
        "controller_xxmi_mapped": first(a_items, "controller", "controller_xxmi_mapped"),
        "unhook_library_returned": first(a_items, "controller", "unhook_library_returned"),
        "child_exit_codes": {stage: first(a_items, "controller", stage) for stage in ["bystander_exit", "target_early_exit", "target_late_exit", "target_early_repeat_exit", "target_late_repeat_exit", "target_after_unhook_exit"]},
        "kept_logs": {path.name: {"bytes": path.stat().st_size, "initialized": "D3D11 DLL successfully initialized" in path.read_text(errors="replace")} for path in sorted((root / "logs").glob("*.txt"))},
        "bystander": {"mapped_before_window": first(a_items, "bystander", "xxmi_mapped_before_window"), "mapped_after_window": first(a_items, "bystander", "xxmi_mapped_after_window"), "completed": first(a_items, "bystander", "bystander_completed") is not None},
        "targets_prefix_a": target_cases,
        "cross_prefix_b": {label: {"mapped_after_window": first(items, "target-early", "xxmi_mapped_after_window"), "device_vtable_module": first(items, "target-early", "device_vtable_module"), "device_create_buffer_module": first(items, "target-early", "device_create_buffer_module"), "completed": first(items, "target-early", "target_completed") is not None} for label, items in b_items.items()},
        "loaddll_xxmi_prefix_a": a_maps,
        "loaddll_xxmi_prefix_b": b_maps,
        "prefix_a_pids_outside_fixture_that_mapped_xxmi": other_a_pids,
        "final_log_initialized": "D3D11 DLL successfully initialized" in body,
        "final_log_bytes": len(body.encode()),
        "real_game_started": False,
        "game_prefix_entered": False,
        "inject_called": False,
        "original_runtime_unchanged": experiment.inventory(source / "ZZMI") == original_tree,
    }
    experiment.save(root / "report.json", report)
    print(json.dumps({"status": "hook_isolation_finished", "report": str(root / "report.json")}), flush=True)


if __name__ == "__main__":
    main()
