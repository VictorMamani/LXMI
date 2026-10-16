/* LXMI mod loader: official XXMI HookLibrary/UnhookLibrary for one new game process.
 *
 * Validated on ZZZ + Steam + Proton (2026-10-15): the global WH_CBT hook stays in
 * the wineserver that runs this helper; ZZMI's DllMain rejects every process that
 * is neither the [Loader] target nor this loader. The hook is removed as soon as
 * the module is seen, by exact path, in the newly created target, or on timeout.
 *
 * Usage: "LXMI Loader.exe" hook-wait <Windows path of the game executable>
 * The module and 3dmloader.dll are derived from this image's directory; no PID,
 * DLL path or process name is accepted from argv. No privileges, suspension,
 * Inject, or process termination. One JSON object per stdout line.
 */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <tlhelp32.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

#define CAP 4096
#define WAIT_FOR_GAME_MS 300000
#define WAIT_FOR_MODULE_MS 30000
#define HEALTH_WINDOW_MS 15000

typedef int (__cdecl *HookLibraryFn)(LPCWSTR, HHOOK *, HANDLE *);
typedef int (__cdecl *UnhookLibraryFn)(HHOOK *, HANDLE *);

static HMODULE loader;
static HHOOK hook;
static HANDLE hook_mutex;
static UnhookLibraryFn unhook_library;

static int fail(const char *status, DWORD error) {
    printf("{\"status\":\"%s\",\"windows_error\":%lu}\n", status, error);
    fflush(stdout);
    return 1;
}

static void unhook(void) {
    if (!unhook_library) return;
    int code = unhook_library(&hook, &hook_mutex);
    unhook_library = NULL;
    printf("{\"status\":\"unhook_library_returned\",\"code\":%d}\n", code);
    fflush(stdout);
}

static unsigned long long now_filetime(void) {
    FILETIME now;
    GetSystemTimeAsFileTime(&now);
    return ((unsigned long long)now.dwHighDateTime << 32) | now.dwLowDateTime;
}

/* Steam maps libraries to drive letters (S:, Z:, ...) that can name the same
 * file: compare filesystem identity rather than path text or basename. */
static BOOL same_file(const wchar_t *a_path, const wchar_t *b_path) {
    HANDLE a = CreateFileW(a_path, FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL, OPEN_EXISTING, 0, NULL);
    HANDLE b = CreateFileW(b_path, FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL, OPEN_EXISTING, 0, NULL);
    BY_HANDLE_FILE_INFORMATION ai, bi;
    BOOL same = a != INVALID_HANDLE_VALUE && b != INVALID_HANDLE_VALUE &&
        GetFileInformationByHandle(a, &ai) && GetFileInformationByHandle(b, &bi) &&
        ai.dwVolumeSerialNumber == bi.dwVolumeSerialNumber &&
        ai.nFileIndexHigh == bi.nFileIndexHigh && ai.nFileIndexLow == bi.nFileIndexLow;
    if (a != INVALID_HANDLE_VALUE) CloseHandle(a);
    if (b != INVALID_HANDLE_VALUE) CloseHandle(b);
    return same;
}

static BOOL bind_image(HANDLE process, const wchar_t *expected, unsigned long long *created) {
    wchar_t path[CAP];
    DWORD count = CAP, code;
    FILETIME c, e, k, u;
    if (!QueryFullProcessImageNameW(process, 0, path, &count) || !same_file(path, expected) ||
        !GetExitCodeProcess(process, &code) || code != STILL_ACTIVE ||
        !GetProcessTimes(process, &c, &e, &k, &u)) return FALSE;
    *created = ((unsigned long long)c.dwHighDateTime << 32) | c.dwLowDateTime;
    return TRUE;
}

/* Exact-path membership in the bound target only. Never unloads anything. */
static int module_present(DWORD pid, const wchar_t *path) {
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
    if (snapshot == INVALID_HANDLE_VALUE) return -1;
    MODULEENTRY32W entry;
    memset(&entry, 0, sizeof(entry));
    entry.dwSize = sizeof(entry);
    int present = 0;
    if (Module32FirstW(snapshot, &entry)) {
        do { if (!_wcsicmp(entry.szExePath, path)) present = 1; }
        while (Module32NextW(snapshot, &entry));
    } else {
        present = -1;
    }
    CloseHandle(snapshot);
    return present;
}

int wmain(int argc, wchar_t **argv) {
    if (argc != 3 || wcscmp(argv[1], L"hook-wait") || wcslen(argv[2]) >= CAP)
        return fail("invalid_arguments", 0);
    const wchar_t *expected_image = argv[2];
    const wchar_t *expected_base = wcsrchr(expected_image, L'\\');
    if (!expected_base || !expected_base[1]) return fail("invalid_arguments", 0);
    expected_base++;
    if (GetFileAttributesW(expected_image) == INVALID_FILE_ATTRIBUTES)
        return fail("game_executable_not_visible", GetLastError());

    wchar_t root[CAP], module_path[CAP], loader_path[CAP];
    DWORD length = GetModuleFileNameW(NULL, root, CAP);
    if (!length || length >= CAP || !wcsrchr(root, L'\\')) return fail("invalid_layout", 0);
    *wcsrchr(root, L'\\') = 0;
    if (swprintf(module_path, CAP, L"%ls\\ZZMI\\d3d11.dll", root) < 0 ||
        swprintf(loader_path, CAP, L"%ls\\3dmloader.dll", root) < 0)
        return fail("path_too_long", 0);

    loader = LoadLibraryExW(loader_path, NULL, LOAD_WITH_ALTERED_SEARCH_PATH);
    if (!loader) return fail("loader_load_failed", GetLastError());
    FARPROC hook_symbol = GetProcAddress(loader, "HookLibrary");
    FARPROC unhook_symbol = GetProcAddress(loader, "UnhookLibrary");
    if (!hook_symbol || !unhook_symbol) return fail("hook_exports_missing", 0);
    HookLibraryFn hook_library = NULL;
    memcpy(&hook_library, &hook_symbol, sizeof(hook_library));
    memcpy(&unhook_library, &unhook_symbol, sizeof(unhook_library));

    unsigned long long armed_at = now_filetime();
    int hooked = hook_library(module_path, &hook, &hook_mutex);
    printf("{\"status\":\"hook_library_returned\",\"code\":%d}\n", hooked);
    fflush(stdout);
    if (hooked != 0) { unhook_library = NULL; return 1; }
    printf("{\"status\":\"armed_hook_waiting_for_new_game\",\"timeout_seconds\":%d}\n", WAIT_FOR_GAME_MS / 1000);
    fflush(stdout);

    DWORD pid = 0, matches = 0;
    HANDLE target = NULL;
    unsigned long long created = 0;
    DWORD started = GetTickCount();
    do {
        HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if (snapshot == INVALID_HANDLE_VALUE) { unhook(); return fail("snapshot_failed", GetLastError()); }
        PROCESSENTRY32W entry;
        memset(&entry, 0, sizeof(entry));
        entry.dwSize = sizeof(entry);
        matches = 0;
        if (Process32FirstW(snapshot, &entry)) {
            do {
                if (_wcsicmp(entry.szExeFile, expected_base)) continue;
                ++matches;
                HANDLE candidate = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, FALSE, entry.th32ProcessID);
                unsigned long long time = 0;
                if (candidate && bind_image(candidate, expected_image, &time)) {
                    if (target) CloseHandle(target);
                    target = candidate;
                    pid = entry.th32ProcessID;
                    created = time;
                } else if (candidate) {
                    CloseHandle(candidate);
                }
            } while (Process32NextW(snapshot, &entry));
        }
        CloseHandle(snapshot);
        if (matches) break;
        Sleep(10);
    } while (GetTickCount() - started < WAIT_FOR_GAME_MS);

    if (target && created <= armed_at) {
        unhook(); CloseHandle(target);
        return fail("existing_game_rejected", 0);
    }
    if (matches != 1 || !target) {
        unhook();
        if (target) CloseHandle(target);
        return fail(matches > 1 ? "ambiguous_target" : "game_not_started", 0);
    }

    /* Same exact-path check as upstream WaitForInjection, bound to this PID. */
    int seen = 0;
    DWORD waiting = GetTickCount();
    while (GetTickCount() - waiting < WAIT_FOR_MODULE_MS) {
        if (WaitForSingleObject(target, 0) == WAIT_OBJECT_0) break;
        seen = module_present(pid, module_path);
        if (seen == 1) break;
        Sleep(5);
    }
    unhook();
    printf("{\"status\":\"game_observed\",\"windows_pid\":%lu,\"mods_module_present\":%d,\"process_age_ms\":%llu}\n",
           pid, seen, (now_filetime() - created) / 10000);
    fflush(stdout);

    int rc = 2;
    if (seen == 1) {
        /* Observe the original handle only; never follow a replacement by name. */
        DWORD observed = WaitForSingleObject(target, HEALTH_WINDOW_MS);
        if (observed == WAIT_TIMEOUT) {
            printf("{\"status\":\"game_alive_after_load\",\"observation_ms\":%d}\n", HEALTH_WINDOW_MS);
            rc = 0;
        } else {
            DWORD exit_code = 0;
            if (observed == WAIT_OBJECT_0 && GetExitCodeProcess(target, &exit_code))
                printf("{\"status\":\"game_exited_after_load\",\"exit_code\":%lu,\"exit_code_hex\":\"0x%08lx\"}\n", exit_code, exit_code);
            else
                printf("{\"status\":\"game_health_unknown\",\"windows_error\":%lu}\n", GetLastError());
            rc = 3;
        }
        fflush(stdout);
    }
    CloseHandle(target);
    FreeLibrary(loader);
    return rc;
}
