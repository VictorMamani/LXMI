/* Host-bound, opt-in ZZZ experiment. Uses the official Inject export, or in
 * hook-wait mode the official HookLibrary/UnhookLibrary pair (global WH_CBT,
 * scoped to this wineserver; ZZMI's DllMain rejects non-target processes).
 * No privileges, arbitrary PID, game launch, or process termination. */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <tlhelp32.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

#define CAP 4096
typedef int (__cdecl *InjectFn)(DWORD, LPCWSTR, int);
typedef int (__cdecl *HookLibraryFn)(LPCWSTR, HHOOK *, HANDLE *);
typedef int (__cdecl *UnhookLibraryFn)(HHOOK *, HANDLE *);
static const wchar_t expected_image[] =
    L"Z:\\media\\university\\Steam\\SteamLibrary\\steamapps\\common\\Zenless Zone Zero\\games\\ZenlessZoneZero Game\\ZenlessZoneZero.exe";

static int result(const char *status, DWORD error) {
    printf("{\"status\":\"%s\",\"windows_error\":%lu}\n", status, error);
    fflush(stdout);
    return 1;
}

static BOOL image_matches(HANDLE process, unsigned long long *created) {
    wchar_t path[CAP]; DWORD count = CAP, code;
    FILETIME c, e, k, u;
    if (!QueryFullProcessImageNameW(process, 0, path, &count)) return FALSE;
    /* Steam assigns S: to this library; Z: and S: can name the same file.
       Compare filesystem identity rather than accepting a basename match. */
    HANDLE expected = CreateFileW(expected_image, FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL, OPEN_EXISTING, 0, NULL);
    HANDLE actual = CreateFileW(path, FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL, OPEN_EXISTING, 0, NULL);
    BY_HANDLE_FILE_INFORMATION a, b;
    BOOL same = expected != INVALID_HANDLE_VALUE && actual != INVALID_HANDLE_VALUE &&
        GetFileInformationByHandle(expected, &a) && GetFileInformationByHandle(actual, &b) &&
        a.dwVolumeSerialNumber == b.dwVolumeSerialNumber &&
        a.nFileIndexHigh == b.nFileIndexHigh && a.nFileIndexLow == b.nFileIndexLow;
    if (expected != INVALID_HANDLE_VALUE) CloseHandle(expected);
    if (actual != INVALID_HANDLE_VALUE) CloseHandle(actual);
    if (!same || !GetExitCodeProcess(process, &code) || code != STILL_ACTIVE ||
        !GetProcessTimes(process, &c, &e, &k, &u)) return FALSE;
    *created = ((unsigned long long)c.dwHighDateTime << 32) | c.dwLowDateTime;
    return TRUE;
}

/* Inspect modules of the already-bound target only. Never unload a module. */
static int module_present(DWORD pid, const wchar_t *path) {
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
    if (snapshot == INVALID_HANDLE_VALUE) return -1;
    MODULEENTRY32W entry; memset(&entry, 0, sizeof(entry)); entry.dwSize = sizeof(entry);
    int present = 0;
    if (Module32FirstW(snapshot, &entry)) {
        do { if (!_wcsicmp(entry.szExePath, path)) present = 1; }
        while (Module32NextW(snapshot, &entry));
    } else { present = -1; }
    CloseHandle(snapshot);
    return present;
}

static BOOL graphics_libraries_present(DWORD pid) {
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
    if (snapshot == INVALID_HANDLE_VALUE) return FALSE;
    MODULEENTRY32W entry; memset(&entry, 0, sizeof(entry)); entry.dwSize = sizeof(entry);
    BOOL d3d = FALSE, dxgi = FALSE;
    if (Module32FirstW(snapshot, &entry)) {
        do {
            if (!_wcsicmp(entry.szModule, L"d3d11.dll")) d3d = TRUE;
            if (!_wcsicmp(entry.szModule, L"dxgi.dll")) dxgi = TRUE;
        } while (Module32NextW(snapshot, &entry));
    }
    CloseHandle(snapshot);
    return d3d && dxgi;
}

int wmain(int argc, wchar_t **argv) {
    if (argc != 2 || (wcscmp(argv[1], L"probe") && wcscmp(argv[1], L"inject") &&
        wcscmp(argv[1], L"wait-inject") && wcscmp(argv[1], L"hook-wait")))
        return result("invalid_mode", 0);
    BOOL hook_mode = !wcscmp(argv[1], L"hook-wait");
    BOOL early = hook_mode || !wcscmp(argv[1], L"wait-inject");
    BOOL execute = early || !wcscmp(argv[1], L"inject");
    FILETIME now; GetSystemTimeAsFileTime(&now);
    unsigned long long armed_at = ((unsigned long long)now.dwHighDateTime << 32) | now.dwLowDateTime;
    wchar_t root[CAP], dll[CAP], loader_path[CAP];
    DWORD length = GetModuleFileNameW(NULL, root, CAP);
    if (!length || length >= CAP || !wcsrchr(root, L'\\')) return result("invalid_layout", 0);
    *wcsrchr(root, L'\\') = 0;
    if (swprintf(dll, CAP, L"%ls\\ZZMI\\d3d11.dll", root) < 0 ||
        swprintf(loader_path, CAP, L"%ls\\3dmloader.dll", root) < 0)
        return result("path_too_long", 0);
    DWORD pid = 0, matches = 0; HANDLE target = NULL;
    unsigned long long created = 0;
    HMODULE hook_loader = NULL; HHOOK hook = NULL; HANDLE hook_mutex = NULL;
    UnhookLibraryFn unhook_library = NULL;
    if (hook_mode) {
        /* Installed before the game exists so ZZMI attaches at its first CBT
           event, ahead of device creation. Removed on every exit path below. */
        hook_loader = LoadLibraryExW(loader_path, NULL, LOAD_WITH_ALTERED_SEARCH_PATH);
        if (!hook_loader) return result("loader_load_failed", GetLastError());
        FARPROC hook_symbol = GetProcAddress(hook_loader, "HookLibrary");
        FARPROC unhook_symbol = GetProcAddress(hook_loader, "UnhookLibrary");
        HookLibraryFn hook_library = NULL;
        if (!hook_symbol || !unhook_symbol) { FreeLibrary(hook_loader); return result("hook_exports_missing", 0); }
        memcpy(&hook_library, &hook_symbol, sizeof(hook_library));
        memcpy(&unhook_library, &unhook_symbol, sizeof(unhook_library));
        int hooked = hook_library(dll, &hook, &hook_mutex);
        printf("{\"status\":\"hook_library_returned\",\"code\":%d}\n", hooked); fflush(stdout);
        if (hooked != 0) { FreeLibrary(hook_loader); return 1; }
    }
#define UNHOOK() do { if (hook_mode && unhook_library) { \
        int unhooked = unhook_library(&hook, &hook_mutex); unhook_library = NULL; \
        printf("{\"status\":\"unhook_library_returned\",\"code\":%d}\n", unhooked); fflush(stdout); } } while (0)
    DWORD started = GetTickCount();
    if (early) { puts(hook_mode ? "{\"status\":\"armed_hook_waiting_for_new_zzz\",\"timeout_seconds\":300}" : "{\"status\":\"armed_waiting_for_new_zzz\",\"timeout_seconds\":300}"); fflush(stdout); }
    do {
        HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if (snapshot == INVALID_HANDLE_VALUE) return result("snapshot_failed", GetLastError());
        PROCESSENTRY32W entry; memset(&entry, 0, sizeof(entry)); entry.dwSize = sizeof(entry);
        matches = 0;
        if (Process32FirstW(snapshot, &entry)) {
            do {
                if (_wcsicmp(entry.szExeFile, L"ZenlessZoneZero.exe")) continue;
                ++matches;
                HANDLE candidate = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, FALSE, entry.th32ProcessID);
                unsigned long long time = 0;
                if (candidate && image_matches(candidate, &time)) {
                    if (target) CloseHandle(target);
                    target = candidate; pid = entry.th32ProcessID; created = time;
                } else if (candidate) CloseHandle(candidate);
            } while (Process32NextW(snapshot, &entry));
        }
        CloseHandle(snapshot);
        if (matches || !early) break;
        Sleep(10);
    } while (GetTickCount() - started < 300000);
    if (early && target && created <= armed_at) {
        UNHOOK(); CloseHandle(target); return result("existing_game_rejected", 0);
    }
    if (matches != 1 || !target) {
        UNHOOK();
        if (target) CloseHandle(target);
        return result(matches > 1 ? "ambiguous_target" : "exact_target_not_visible", 0);
    }
    if (hook_mode) {
        /* Same exact-path check as upstream WaitForInjection, bound to this PID.
           Record whether d3d11/dxgi were already mapped when ZZMI appeared. */
        int seen = 0; BOOL graphics_at_load = FALSE;
        DWORD waiting = GetTickCount();
        while (GetTickCount() - waiting < 30000) {
            if (WaitForSingleObject(target, 0) == WAIT_OBJECT_0) break;
            seen = module_present(pid, dll);
            if (seen == 1) { graphics_at_load = graphics_libraries_present(pid); break; }
            Sleep(5);
        }
        UNHOOK();
        GetSystemTimeAsFileTime(&now);
        unsigned long long current = ((unsigned long long)now.dwHighDateTime << 32) | now.dwLowDateTime;
        printf("{\"status\":\"hook_target_observed\",\"windows_pid\":%lu,\"creation_time\":\"%llu\",\"exact_module_present\":%d,\"graphics_libraries_already_mapped\":%s,\"process_age_ms\":%llu,\"visual_result\":\"pending_user\"}\n",
               pid, created, seen, graphics_at_load ? "true" : "false", (current - created) / 10000);
        fflush(stdout);
        BOOL healthy = FALSE;
        if (seen == 1) {
            DWORD observed = WaitForSingleObject(target, 15000);
            if (observed == WAIT_TIMEOUT) {
                healthy = TRUE;
                puts("{\"status\":\"original_game_alive_after_load\",\"observation_ms\":15000}");
            } else {
                DWORD exit_code = 0;
                if (observed == WAIT_OBJECT_0 && GetExitCodeProcess(target, &exit_code))
                    printf("{\"status\":\"original_game_exited_after_load\",\"exit_code\":%lu,\"exit_code_hex\":\"0x%08lx\"}\n", exit_code, exit_code);
                else
                    printf("{\"status\":\"original_game_health_unknown\",\"windows_error\":%lu}\n", GetLastError());
            }
            fflush(stdout);
        }
        FreeLibrary(hook_loader); CloseHandle(target);
        return seen == 1 ? (healthy ? 0 : 3) : 2;
    }
    int present = module_present(pid, dll);
    if (early && present < 0) {
        DWORD waiting = GetTickCount();
        do { Sleep(1); present = module_present(pid, dll); }
        while (present < 0 && GetTickCount() - waiting < 1000);
    }
    printf("{\"status\":\"target_bound\",\"windows_pid\":%lu,\"creation_time\":\"%llu\",\"exact_image\":true,\"module_already_present\":%d,\"inject_requested\":%s}\n",
           pid, created, present, execute ? "true" : "false");
    fflush(stdout);
    if (!execute) { CloseHandle(target); return 0; }
    if (present != 0) { CloseHandle(target); return result("module_present_or_unreadable", 0); }
    if (early) {
        /* DllMain requires original d3d11 and dxgi modules to be mapped.
           Do not suspend the game or modify its execution/graphics settings. */
        DWORD waiting = GetTickCount();
        while (!graphics_libraries_present(pid) && GetTickCount() - waiting < 10000) {
            if (WaitForSingleObject(target, 0) == WAIT_OBJECT_0) {
                CloseHandle(target); return result("game_exited_before_graphics", 0);
            }
            Sleep(1);
        }
        if (!graphics_libraries_present(pid)) { CloseHandle(target); return result("graphics_libraries_timeout", 0); }
        GetSystemTimeAsFileTime(&now);
        unsigned long long current = ((unsigned long long)now.dwHighDateTime << 32) | now.dwLowDateTime;
        printf("{\"status\":\"early_graphics_libraries_present\",\"process_age_ms\":%llu}\n", (current - created) / 10000);
        fflush(stdout);
    }
    unsigned long long rechecked = 0;
    if (!image_matches(target, &rechecked) || created != rechecked) {
        CloseHandle(target); return result("target_changed", 0);
    }
    /* Keep this handle open through Inject so the PID cannot be recycled. */
    HMODULE loader = LoadLibraryExW(loader_path, NULL, LOAD_WITH_ALTERED_SEARCH_PATH);
    if (!loader) { DWORD error = GetLastError(); CloseHandle(target); return result("loader_load_failed", error); }
    FARPROC symbol = GetProcAddress(loader, "Inject"); InjectFn inject = NULL;
    if (!symbol || sizeof(symbol) != sizeof(inject)) {
        FreeLibrary(loader); CloseHandle(target); return result("inject_export_missing", 0);
    }
    memcpy(&inject, &symbol, sizeof(inject));
    puts("{\"status\":\"calling_official_inject\",\"timeout_seconds\":15}"); fflush(stdout);
    int code = inject(pid, dll, 15);
    present = module_present(pid, dll);
    printf("{\"status\":\"inject_returned\",\"inject_code\":%d,\"exact_module_present\":%d,\"visual_result\":\"pending_user\"}\n", code, present);
    fflush(stdout);
    BOOL healthy = FALSE;
    if (code == 0 && present == 1) {
        /* Injection returns before the game's graphics initialization finishes.
           Observe the original process handle, never a replacement by name. */
        DWORD observed = WaitForSingleObject(target, 15000);
        if (observed == WAIT_TIMEOUT) {
            healthy = TRUE;
            puts("{\"status\":\"original_game_alive_after_load\",\"observation_ms\":15000}");
        } else if (observed == WAIT_OBJECT_0) {
            DWORD exit_code = 0;
            if (GetExitCodeProcess(target, &exit_code))
                printf("{\"status\":\"original_game_exited_after_load\",\"exit_code\":%lu,\"exit_code_hex\":\"0x%08lx\"}\n", exit_code, exit_code);
            else
                printf("{\"status\":\"original_game_health_unknown\",\"windows_error\":%lu}\n", GetLastError());
        } else {
            printf("{\"status\":\"original_game_health_unknown\",\"windows_error\":%lu}\n", GetLastError());
        }
        fflush(stdout);
    }
    FreeLibrary(loader); CloseHandle(target);
    return code == 0 && present == 1 ? (healthy ? 0 : 3) : 2;
}
