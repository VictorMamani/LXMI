/* LXMI-owned named-event probe. No process enumeration, process handles or DLL loading. */
#include <windows.h>
#include <stdio.h>
#include <wchar.h>
#ifndef PEER_A
#define PEER_A 0
#endif
static void json_string(const char *text) {
    putchar('"');
    for (const unsigned char *p = (const unsigned char *)text; *p; ++p) {
        if (*p == '"' || *p == '\\') printf("\\%c", *p);
        else if (*p < 32) printf("\\u%04x", *p);
        else putchar(*p);
    }
    putchar('"');
}
static void env_json(const char *key) {
    char value[4096] = {0};
    DWORD n = GetEnvironmentVariableA(key, value, sizeof(value));
    if (n >= sizeof(value)) value[0] = 0;
    json_string(value);
}
int wmain(int argc, wchar_t **argv) {
    if (argc == 2 && !wcscmp(argv[1], L"bootstrap")) {
        puts("{\"protocol\":1,\"bootstrap\":true}"); return 0;
    }
    if (argc != 4 || wcslen(argv[1]) != 64) return 2;
    for (size_t i = 0; i < 64; ++i)
        if (!((argv[1][i] >= L'0' && argv[1][i] <= L'9') ||
              (argv[1][i] >= L'a' && argv[1][i] <= L'f'))) return 2;
    /* Harness supplies only nonce-scoped LXMI marker paths through direct argv. */
    HANDLE marker = CreateFileW(argv[2], GENERIC_READ, FILE_SHARE_READ, NULL,
                                OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
    char contents[65] = {0}; DWORD count = 0;
    BOOL path_readable = marker != INVALID_HANDLE_VALUE &&
        ReadFile(marker, contents, 64, &count, NULL) && count == 64;
    if (marker != INVALID_HANDLE_VALUE) CloseHandle(marker);
    char nonce[65]; for (int i = 0; i < 64; ++i) nonce[i] = (char)argv[1][i]; nonce[64] = 0;
    path_readable = path_readable && !strcmp(contents, nonce);
    wchar_t name[100]; swprintf(name, 100, L"Local\\LXMIContextV1-%ls", argv[1]);
    HANDLE event; DWORD error = 0; BOOL probe = FALSE;
    if (PEER_A) {
        event = CreateEventW(NULL, TRUE, FALSE, name); error = GetLastError();
        if (event && error != ERROR_ALREADY_EXISTS && path_readable) {
            HANDLE ready = CreateFileW(argv[3], GENERIC_WRITE, 0, NULL, CREATE_NEW,
                                       FILE_ATTRIBUTE_NORMAL, NULL);
            if (ready != INVALID_HANDLE_VALUE) {
                DWORD written; WriteFile(ready, nonce, 64, &written, NULL); CloseHandle(ready);
                probe = WaitForSingleObject(event, 12000) == WAIT_OBJECT_0;
            }
        }
    } else {
        event = OpenEventW(EVENT_MODIFY_STATE, FALSE, name); error = GetLastError();
        if (event && path_readable) probe = SetEvent(event);
    }
    if (event) CloseHandle(event);
    printf("{\"protocol\":1,\"peer\":\"%s\",\"pid\":%lu,\"nonce\":", PEER_A ? "a" : "b", GetCurrentProcessId());
    json_string(nonce);
    printf(",\"path_readable\":%s,\"probe_succeeded\":%s,\"windows_error\":%lu,\"wine_prefix\":",
           path_readable ? "true" : "false", probe ? "true" : "false", error);
    env_json("WINEPREFIX"); printf(",\"compatdata\":"); env_json("STEAM_COMPAT_DATA_PATH");
    printf(",\"wineserver_binary\":"); env_json("WINESERVER"); puts("}");
    return path_readable ? 0 : 3;
}
