#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
#include <wchar.h>

#define PATH_CAP 4096

static int is_nonce(const wchar_t *value) {
    size_t index;
    if (!value || wcslen(value) != 32) return 0;
    for (index = 0; index < 32; ++index) {
        if (!((value[index] >= L'0' && value[index] <= L'9') ||
              (value[index] >= L'a' && value[index] <= L'f'))) return 0;
    }
    return 1;
}

static int path_for(const wchar_t *filename, wchar_t *output, size_t capacity) {
    wchar_t executable[PATH_CAP];
    wchar_t *last;
    DWORD count = GetModuleFileNameW(NULL, executable, (DWORD)(sizeof(executable) / sizeof(executable[0])));
    if (!count || count >= (DWORD)(sizeof(executable) / sizeof(executable[0]))) return 0;
    last = wcsrchr(executable, L'\\');
    if (!last) return 0;
    *last = L'\0';
    last = wcsrchr(executable, L'\\');
    if (!last) return 0;
    *last = L'\0';
    return _snwprintf_s(output, capacity, _TRUNCATE, L"%ls\\results\\%ls", executable, filename) >= 0;
}

static int file_exists(const wchar_t *path) {
    DWORD attributes = GetFileAttributesW(path);
    return attributes != INVALID_FILE_ATTRIBUTES && !(attributes & FILE_ATTRIBUTE_DIRECTORY);
}

static int create_ready_marker(const wchar_t *path, const wchar_t *nonce) {
    char payload[160];
    int length = snprintf(payload, sizeof(payload), "{\"protocol\":1,\"ready\":true,\"nonce\":\"%ls\"}\n", nonce);
    DWORD written = 0;
    HANDLE file;
    if (length <= 0 || (size_t)length >= sizeof(payload)) return 0;
    file = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, NULL);
    if (file == INVALID_HANDLE_VALUE) return 0;
    if (!WriteFile(file, payload, (DWORD)length, &written, NULL) || written != (DWORD)length) {
        CloseHandle(file);
        return 0;
    }
    FlushFileBuffers(file);
    CloseHandle(file);
    return 1;
}

int wmain(void) {
    wchar_t nonce[64] = {0};
    wchar_t phase[32] = {0};
    wchar_t ready_path[PATH_CAP];
    wchar_t result_path[PATH_CAP];
    wchar_t stop_path[PATH_CAP];
    wchar_t process_path[PATH_CAP];
    wchar_t *basename;
    DWORD count;
    DWORD started;

    count = GetModuleFileNameW(NULL, process_path, PATH_CAP);
    if (!count || count >= PATH_CAP) return 2;
    basename = wcsrchr(process_path, L'\\');
    basename = basename ? basename + 1 : process_path;
    if (_wcsicmp(basename, L"lxmi-loader-test-target.exe") != 0) return 3;

    count = GetEnvironmentVariableW(L"LXMI_LOADER_TEST_NONCE", nonce, (DWORD)(sizeof(nonce) / sizeof(nonce[0])));
    if (!count || count >= (DWORD)(sizeof(nonce) / sizeof(nonce[0])) || !is_nonce(nonce)) return 4;
    GetEnvironmentVariableW(L"LXMI_LOADER_TEST_PHASE", phase, (DWORD)(sizeof(phase) / sizeof(phase[0])));
    if (!path_for(L"target-ready.json", ready_path, PATH_CAP) ||
        !path_for(L"loader-result.json", result_path, PATH_CAP) ||
        !path_for(L"stop.json", stop_path, PATH_CAP) ||
        !create_ready_marker(ready_path, nonce)) return 5;

    wprintf(L"READY process=lxmi-loader-test-target.exe\n");
    fflush(stdout);
    started = GetTickCount();
    for (;;) {
        if (file_exists(stop_path) || file_exists(result_path)) {
            wprintf(L"EXIT clean\n");
            fflush(stdout);
            return 0;
        }
        if (_wcsicmp(phase, L"baseline") == 0 && GetTickCount() - started >= 300) {
            wprintf(L"EXIT baseline\n");
            fflush(stdout);
            return 0;
        }
        if (GetTickCount() - started >= 8000) {
            fwprintf(stderr, L"ERROR target timeout\n");
            fflush(stderr);
            return 6;
        }
        Sleep(25);
    }
}
