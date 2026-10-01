/* Session-bound pipe identity probe; no process handles or memory access. */
typedef struct { DWORD pid; char nonce[33]; } ContextHello;
static void context_env(const char *key) {
    char value[4096] = {0};
    DWORD n = GetEnvironmentVariableA(key, value, sizeof(value));
    if (n >= sizeof(value)) value[0] = 0;
    putchar('"');
    for (char *p = value; *p; ++p) {
        if (*p == '\\' || *p == '"') putchar('\\');
        if ((unsigned char)*p >= 32) putchar(*p);
    }
    putchar('"');
}
static int context_layout(wchar_t *root, const wchar_t *nonce) {
    DWORD n = GetModuleFileNameW(NULL, root, PATH_CAP);
    if (!n || n >= PATH_CAP || !valid_nonce(nonce)) return 0;
    wchar_t *p = wcsrchr(root, L'\\');
    if (!p) return 0;
    *p = 0; p = wcsrchr(root, L'\\'); if (!p) return 0;
    *p = 0; p = wcsrchr(root, L'\\');
    if (!p || wcscmp(p + 1, L"loader-v1")) return 0;
    wchar_t check[PATH_CAP]; wcscpy(check, root);
    p = wcsrchr(check, L'\\'); *p = 0; p = wcsrchr(check, L'\\');
    if (!p || wcscmp(p + 1, nonce)) return 0;
    *p = 0; p = wcsrchr(check, L'\\');
    if (!p || wcscmp(p + 1, L"direct-context-v1")) return 0;
    *p = 0; p = wcsrchr(check, L'\\');
    if (!p || wcscmp(p + 1, L"tests")) return 0;
    *p = 0; p = wcsrchr(check, L'\\');
    if (!p || wcscmp(p + 1, L"lxmi")) return 0;
    return 1;
}
#ifndef CONTEXT_RUNNER
static int context_target(const wchar_t *nonce) {
    wchar_t root[PATH_CAP], ready[PATH_CAP], stop[PATH_CAP], marker[PATH_CAP], pipe_name[128];
    wchar_t executable[PATH_CAP];
    GetModuleFileNameW(NULL, executable, PATH_CAP);
    wchar_t *base = wcsrchr(executable, L'\\');
    if (!base || wcscmp(base + 1, L"lxmi-loader-test-target.exe") || !context_layout(root, nonce)) return 20;
    swprintf(ready, PATH_CAP, L"%ls\\results\\target-ready", root);
    swprintf(stop, PATH_CAP, L"%ls\\results\\stop", root);
    swprintf(marker, PATH_CAP, L"%ls\\results\\filesystem-marker", root);
    swprintf(pipe_name, 128, L"\\\\.\\pipe\\LXMIInjectContext-%ls", nonce);
    ContextHello hello = {0}; hello.pid = GetCurrentProcessId();
    for (int i = 0; i < 32; ++i) hello.nonce[i] = (char)nonce[i];
    char data[33] = {0}; DWORD count = 0;
    HANDLE f = CreateFileW(marker, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    BOOL readable = f != INVALID_HANDLE_VALUE && ReadFile(f, data, 32, &count, NULL) && count == 32 && !strcmp(data, hello.nonce);
    if (f != INVALID_HANDLE_VALUE) CloseHandle(f);
    HANDLE pipe = CreateNamedPipeW(pipe_name, PIPE_ACCESS_OUTBOUND | FILE_FLAG_FIRST_PIPE_INSTANCE,
                                   PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_NOWAIT, 1, 256, 256, 0, NULL);
    if (pipe == INVALID_HANDLE_VALUE || !readable) return 21;
    f = CreateFileW(ready, GENERIC_WRITE, 0, NULL, CREATE_NEW, 0, NULL);
    if (f == INVALID_HANDLE_VALUE) { CloseHandle(pipe); return 22; }
    WriteFile(f, hello.nonce, 32, &count, NULL); CloseHandle(f);
    DWORD start = GetTickCount(); BOOL sent = FALSE;
    while (GetTickCount() - start < 30000 && !exists(stop)) {
        if (!sent) {
            BOOL connected = ConnectNamedPipe(pipe, NULL);
            DWORD err = GetLastError();
            if (connected || err == ERROR_PIPE_CONNECTED)
                sent = WriteFile(pipe, &hello, sizeof(hello), &count, NULL) && count == sizeof(hello);
        }
        Sleep(25);
    }
    CloseHandle(pipe);
    printf("{\"protocol\":1,\"pid\":%lu,\"nonce\":\"%s\",\"path_readable\":%s,\"handshake_sent\":%s,\"wine_prefix\":", hello.pid, hello.nonce, readable ? "true" : "false", sent ? "true" : "false");
    context_env("WINEPREFIX"); printf(",\"compatdata\":"); context_env("STEAM_COMPAT_DATA_PATH"); puts("}");
    return 0;
}

#endif
