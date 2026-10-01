/* Only consumes the nonce-scoped pipe of the owned fixture. Never accepts a PID. */
#include <windows.h>
#include <stdio.h>
#include <wchar.h>
#include <string.h>
#define PATH_CAP 4096
typedef int (__cdecl *UpstreamInject)(DWORD, LPCWSTR, int);
static int valid_nonce(const wchar_t *nonce) {
    if (wcslen(nonce) != 32) return 0;
    for (int i = 0; i < 32; ++i)
        if (!((nonce[i] >= L'0' && nonce[i] <= L'9') || (nonce[i] >= L'a' && nonce[i] <= L'f'))) return 0;
    return 1;
}
#define CONTEXT_RUNNER 1
#include "context_target.h"
int wmain(int argc, wchar_t **argv) {
    if (argc == 2 && !wcscmp(argv[1], L"bootstrap")) { puts("{\"protocol\":1,\"bootstrap\":true}"); return 0; }
    wchar_t root[PATH_CAP], name[128], dll[PATH_CAP], loader_path[PATH_CAP], marker[PATH_CAP];
    if (argc != 2 || !context_layout(root, argv[1])) return 20;
    swprintf(name, 128, L"\\\\.\\pipe\\LXMIInjectContext-%ls", argv[1]);
    swprintf(dll, PATH_CAP, L"%ls\\runtime\\lxmi-loader-test.dll", root);
    swprintf(loader_path, PATH_CAP, L"%ls\\runtime\\3dmloader.dll", root);
    swprintf(marker, PATH_CAP, L"%ls\\results\\filesystem-marker", root);
    char nonce[33] = {0}; for (int i = 0; i < 32; ++i) nonce[i] = (char)argv[1][i];
    char data[33] = {0}; DWORD count = 0;
    HANDLE f = CreateFileW(marker, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    BOOL readable = f != INVALID_HANDLE_VALUE && ReadFile(f, data, 32, &count, NULL) && count == 32 && !strcmp(data, nonce);
    if (f != INVALID_HANDLE_VALUE) CloseHandle(f);
    HANDLE pipe = INVALID_HANDLE_VALUE; DWORD start = GetTickCount(), error = 0;
    while (readable && GetTickCount() - start < 5000) {
        pipe = CreateFileW(name, GENERIC_READ, 0, NULL, OPEN_EXISTING, 0, NULL);
        if (pipe != INVALID_HANDLE_VALUE) break;
        error = GetLastError(); Sleep(25);
    }
    ContextHello hello = {0}; ULONG server_pid = 0; BOOL bound = FALSE; int code = -1;
    if (pipe != INVALID_HANDLE_VALUE) {
        BOOL pid_ok = GetNamedPipeServerProcessId(pipe, &server_pid);
        if (!pid_ok) error = GetLastError();
        start = GetTickCount();
        while (GetTickCount() - start < 2000) {
            if (PeekNamedPipe(pipe, NULL, 0, NULL, &count, NULL) && count >= sizeof(hello)) {
                if (ReadFile(pipe, &hello, sizeof(hello), &count, NULL) && count == sizeof(hello))
                    bound = pid_ok && server_pid == hello.pid && hello.pid != 0 && !memcmp(hello.nonce, nonce, sizeof(hello.nonce));
                break;
            }
            Sleep(25);
        }
        if (!bound && !error) error = GetLastError();
    }
    /* Identity is established before this query-only handle is opened.
       Retain the owned process object so its Windows PID cannot be reused
       while the upstream API resolves it. No memory access rights requested. */
    HANDLE owned = NULL;
    if (bound) {
        owned = OpenProcess(SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, FALSE, server_pid);
        wchar_t actual[PATH_CAP], expected[PATH_CAP]; DWORD capacity = PATH_CAP, exit_code = 0;
        swprintf(expected, PATH_CAP, L"%ls\\target\\lxmi-loader-test-target.exe", root);
        bound = owned && QueryFullProcessImageNameW(owned, 0, actual, &capacity)
            && !_wcsicmp(actual, expected) && GetExitCodeProcess(owned, &exit_code) && exit_code == STILL_ACTIVE;
        if (!bound) error = GetLastError();
    }
    BOOL called = FALSE;
    if (bound && readable) {
        HMODULE loader = LoadLibraryW(loader_path);
        if (loader) {
            FARPROC symbol = GetProcAddress(loader, "Inject"); UpstreamInject inject = NULL;
            if (symbol && sizeof(symbol) == sizeof(inject)) {
                memcpy(&inject, &symbol, sizeof(inject)); called = TRUE;
                code = inject(server_pid, dll, 5);
            } else error = GetLastError();
            FreeLibrary(loader);
        } else error = GetLastError();
    }
    if (owned) CloseHandle(owned);
    if (pipe != INVALID_HANDLE_VALUE) CloseHandle(pipe);
    printf("{\"protocol\":1,\"pid\":%lu,\"target_pid\":%lu,\"nonce\":\"%s\",\"path_readable\":%s,\"identity_bound\":%s,\"inject_called\":%s,\"inject_code\":%d,\"windows_error\":%lu,\"wine_prefix\":", GetCurrentProcessId(), server_pid, nonce, readable ? "true" : "false", bound ? "true" : "false", called ? "true" : "false", code, error);
    context_env("WINEPREFIX"); printf(",\"compatdata\":"); context_env("STEAM_COMPAT_DATA_PATH"); puts("}");
    return 0;
}
