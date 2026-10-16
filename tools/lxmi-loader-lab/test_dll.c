#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

#define PATH_CAP 4096
#define UTF8_CAP (PATH_CAP * 4)
#define JSON_CAP (UTF8_CAP * 5)

static int is_nonce(const wchar_t *value) {
    size_t index;
    if (!value || wcslen(value) != 32) return 0;
    for (index = 0; index < 32; ++index) {
        if (!((value[index] >= L'0' && value[index] <= L'9') ||
              (value[index] >= L'a' && value[index] <= L'f'))) return 0;
    }
    return 1;
}

static int utf8(const wchar_t *input, char *output, int capacity) {
    int count = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, input, -1, output, capacity, NULL, NULL);
    return count > 0 && count < capacity;
}

static int append_json_string(char *output, size_t capacity, size_t *offset, const char *input) {
    const unsigned char *cursor = (const unsigned char *)input;
    if (*offset >= capacity) return 0;
    output[(*offset)++] = '"';
    while (*cursor) {
        char escaped[7];
        size_t length = 0;
        switch (*cursor) {
            case '"': escaped[0] = '\\'; escaped[1] = '"'; length = 2; break;
            case '\\': escaped[0] = '\\'; escaped[1] = '\\'; length = 2; break;
            case '\n': escaped[0] = '\\'; escaped[1] = 'n'; length = 2; break;
            case '\r': escaped[0] = '\\'; escaped[1] = 'r'; length = 2; break;
            case '\t': escaped[0] = '\\'; escaped[1] = 't'; length = 2; break;
            default:
                if (*cursor < 0x20) {
                    int written = snprintf(escaped, sizeof(escaped), "\\u%04x", *cursor);
                    if (written != 6) return 0;
                    length = 6;
                } else {
                    escaped[0] = (char)*cursor;
                    length = 1;
                }
        }
        if (*offset + length >= capacity) return 0;
        memcpy(output + *offset, escaped, length);
        *offset += length;
        ++cursor;
    }
    if (*offset + 1 >= capacity) return 0;
    output[(*offset)++] = '"';
    output[*offset] = '\0';
    return 1;
}

static int result_path(wchar_t *output, size_t capacity, const wchar_t *process_path) {
    wchar_t directory[PATH_CAP];
    wchar_t *last;
    if (wcslen(process_path) >= PATH_CAP) return 0;
    wcscpy_s(directory, PATH_CAP, process_path);
    last = wcsrchr(directory, L'\\');
    if (!last) return 0;
    *last = L'\0';
    last = wcsrchr(directory, L'\\');
    if (!last) return 0;
    *last = L'\0';
    return _snwprintf_s(output, capacity, _TRUNCATE, L"%ls\\results\\loader-result.json", directory) >= 0;
}

static DWORD WINAPI write_marker(void *module) {
    wchar_t nonce[64] = {0};
    wchar_t process_path[PATH_CAP];
    wchar_t dll_path[PATH_CAP];
    wchar_t output_path[PATH_CAP];
    wchar_t *basename;
    char process_utf8[UTF8_CAP];
    char dll_utf8[UTF8_CAP];
    char json[JSON_CAP];
    size_t offset = 0;
    int written;
    DWORD count;
    DWORD bytes_written = 0;
    HANDLE file;

    count = GetModuleFileNameW(NULL, process_path, PATH_CAP);
    if (!count || count >= PATH_CAP) return 1;
    basename = wcsrchr(process_path, L'\\');
    basename = basename ? basename + 1 : process_path;
    if (_wcsicmp(basename, L"lxmi-loader-test-target.exe") != 0) return 2;

    count = GetEnvironmentVariableW(L"LXMI_LOADER_TEST_NONCE", nonce, (DWORD)(sizeof(nonce) / sizeof(nonce[0])));
    if (!count || count >= (DWORD)(sizeof(nonce) / sizeof(nonce[0])) || !is_nonce(nonce)) return 3;
    count = GetModuleFileNameW((HMODULE)module, dll_path, PATH_CAP);
    if (!count || count >= PATH_CAP || !result_path(output_path, PATH_CAP, process_path)) return 4;
    if (!utf8(process_path, process_utf8, UTF8_CAP) || !utf8(dll_path, dll_utf8, UTF8_CAP)) return 5;

    written = snprintf(json, sizeof(json), "{\"protocol\":1,\"loaded\":true,\"nonce\":\"%ls\",\"process\":\"lxmi-loader-test-target.exe\",\"process_path_windows\":", nonce);
    if (written <= 0 || (size_t)written >= sizeof(json)) return 6;
    offset = (size_t)written;
    if (!append_json_string(json, sizeof(json), &offset, process_utf8)) return 7;
    written = snprintf(json + offset, sizeof(json) - offset, ",\"dll_path_windows\":");
    if (written <= 0 || (size_t)written >= sizeof(json) - offset) return 8;
    offset += (size_t)written;
    if (!append_json_string(json, sizeof(json), &offset, dll_utf8)) return 9;
    if (offset + 2 >= sizeof(json)) return 10;
    json[offset++] = '}';
    json[offset++] = '\n';

    file = CreateFileW(output_path, GENERIC_WRITE, 0, NULL, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, NULL);
    if (file == INVALID_HANDLE_VALUE) return 11;
    if (!WriteFile(file, json, (DWORD)offset, &bytes_written, NULL) || bytes_written != (DWORD)offset) {
        CloseHandle(file);
        return 12;
    }
    FlushFileBuffers(file);
    CloseHandle(file);
    return 0;
}

BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID reserved) {
    (void)reserved;
    if (reason == DLL_PROCESS_ATTACH) {
        wchar_t process_path[PATH_CAP];
        wchar_t *basename;
        HANDLE worker;
        DWORD count = GetModuleFileNameW(NULL, process_path, PATH_CAP);
        if (!count || count >= PATH_CAP) return TRUE;
        basename = wcsrchr(process_path, L'\\');
        basename = basename ? basename + 1 : process_path;
        if (_wcsicmp(basename, L"lxmi-loader-test-target.exe") != 0) return TRUE;
        DisableThreadLibraryCalls(instance);
        worker = CreateThread(NULL, 0, write_marker, instance, 0, NULL);
        if (worker) CloseHandle(worker);
    }
    return TRUE;
}
