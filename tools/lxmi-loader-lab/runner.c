#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

#define PATH_CAP 4096
#define TEXT_CAP 65536

typedef int (__cdecl *UpstreamInject)(DWORD, LPCWSTR, int);

typedef struct RunnerResult {
    const char *mode;
    BOOL target_started;
    BOOL target_ready;
    BOOL loader_loaded;
    BOOL inject_called;
    int inject_code;
    int target_exit_code;
    BOOL marker_present;
    BOOL marker_process_matches;
    BOOL nonce_verified;
    char marker_nonce[33];
    const char *diagnostic;
} RunnerResult;

static int valid_nonce(const wchar_t *value) {
    size_t index;
    if (!value || wcslen(value) != 32) return 0;
    for (index = 0; index < 32; ++index) {
        if (!((value[index] >= L'0' && value[index] <= L'9') ||
              (value[index] >= L'a' && value[index] <= L'f'))) return 0;
    }
    return 1;
}

static int append_path(wchar_t *output, size_t capacity, const wchar_t *base, const wchar_t *suffix) {
    int written = _snwprintf_s(output, capacity, _TRUNCATE, L"%ls\\%ls", base, suffix);
    return written >= 0;
}

static int safe_regular_file(const wchar_t *path) {
    DWORD attributes = GetFileAttributesW(path);
    return attributes != INVALID_FILE_ATTRIBUTES &&
           !(attributes & FILE_ATTRIBUTE_DIRECTORY) &&
           !(attributes & FILE_ATTRIBUTE_REPARSE_POINT);
}

static int file_exists(const wchar_t *path) {
    DWORD attributes = GetFileAttributesW(path);
    return attributes != INVALID_FILE_ATTRIBUTES && !(attributes & FILE_ATTRIBUTE_DIRECTORY);
}

static int read_small_file(const wchar_t *path, char *buffer, DWORD capacity, DWORD *length) {
    HANDLE file = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
    LARGE_INTEGER size;
    DWORD read_count = 0;
    if (file == INVALID_HANDLE_VALUE) return 0;
    if (!GetFileSizeEx(file, &size) || size.QuadPart < 0 || (ULONGLONG)size.QuadPart >= capacity) {
        CloseHandle(file);
        return 0;
    }
    if (!ReadFile(file, buffer, (DWORD)size.QuadPart, &read_count, NULL) || read_count != (DWORD)size.QuadPart) {
        CloseHandle(file);
        return 0;
    }
    CloseHandle(file);
    buffer[read_count] = '\0';
    *length = read_count;
    return 1;
}

static int wait_for_exact_file(const wchar_t *path, const char *expected, DWORD timeout_ms) {
    DWORD started = GetTickCount();
    char buffer[512];
    DWORD length;
    while (GetTickCount() - started < timeout_ms) {
        if (read_small_file(path, buffer, sizeof(buffer), &length) && strcmp(buffer, expected) == 0) return 1;
        Sleep(25);
    }
    return 0;
}

static BOOL wait_target(PROCESS_INFORMATION *process, DWORD timeout_ms, int *exit_code) {
    DWORD wait = WaitForSingleObject(process->hProcess, timeout_ms);
    if (wait == WAIT_TIMEOUT) {
        TerminateProcess(process->hProcess, 98);
        WaitForSingleObject(process->hProcess, 1000);
        *exit_code = 98;
        return FALSE;
    }
    if (wait != WAIT_OBJECT_0 || !GetExitCodeProcess(process->hProcess, (DWORD *)exit_code)) {
        *exit_code = -1;
        return FALSE;
    }
    return TRUE;
}

static void write_stop_file(const wchar_t *path) {
    HANDLE file = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (file != INVALID_HANDLE_VALUE) {
        static const char stop[] = "LXMI test stop\n";
        DWORD written = 0;
        WriteFile(file, stop, (DWORD)(sizeof(stop) - 1), &written, NULL);
        FlushFileBuffers(file);
        CloseHandle(file);
    }
}

static void inspect_marker(const wchar_t *path, RunnerResult *result) {
    char buffer[TEXT_CAP];
    DWORD length = 0;
    if (!read_small_file(path, buffer, sizeof(buffer), &length)) return;
    result->marker_present = TRUE;
    result->marker_process_matches = strstr(buffer, "\"process\":\"lxmi-loader-test-target.exe\"") != NULL;
    {
        const char *start = strstr(buffer, "\"nonce\":\"");
        if (start) {
            size_t index;
            start += strlen("\"nonce\":\"");
            for (index = 0; index < 32 && start[index]; ++index) {
                if (!((start[index] >= '0' && start[index] <= '9') || (start[index] >= 'a' && start[index] <= 'f'))) break;
            }
            if (index == 32) {
                memcpy(result->marker_nonce, start, 32);
                result->marker_nonce[32] = '\0';
            }
        }
    }
}

static void emit_result(const RunnerResult *result) {
    const char *marker_nonce = result->marker_nonce[0] ? result->marker_nonce : "";
    const char *diagnostic = result->diagnostic ? "\"" : "";
    const char *diagnostic_value = result->diagnostic ? result->diagnostic : "null";
    const char *diagnostic_suffix = result->diagnostic ? "\"" : "";
    printf("{\"protocol\":1,\"mode\":\"%s\",\"target_started\":%s,\"target_ready\":%s,"
           "\"upstream_loader_loaded\":%s,\"inject_called\":%s,\"upstream_inject_code\":%d,"
           "\"target_exit_code\":%d,\"marker_present\":%s,\"marker_process_matches\":%s,"
           "\"nonce_verified\":%s,\"marker_nonce\":\"%s\",\"diagnostic\":%s%s%s}\n",
           result->mode ? result->mode : "unknown",
           result->target_started ? "true" : "false",
           result->target_ready ? "true" : "false",
           result->loader_loaded ? "true" : "false",
           result->inject_called ? "true" : "false",
           result->inject_code,
           result->target_exit_code,
           result->marker_present ? "true" : "false",
           result->marker_process_matches ? "true" : "false",
           result->nonce_verified ? "true" : "false",
           marker_nonce,
           diagnostic,
           diagnostic_value,
           diagnostic_suffix);
    fflush(stdout);
}

static int derive_layout(wchar_t *root, size_t root_capacity, wchar_t *target, size_t target_capacity,
                         wchar_t *runtime, size_t runtime_capacity, wchar_t *results, size_t results_capacity) {
    wchar_t executable[PATH_CAP];
    wchar_t runner_dir[PATH_CAP];
    wchar_t *last;
    wchar_t *parent_last;
    DWORD count = GetModuleFileNameW(NULL, executable, PATH_CAP);
    if (!count || count >= PATH_CAP) return 0;
    last = wcsrchr(executable, L'\\');
    if (!last) return 0;
    *last = L'\0';
    if (wcslen(executable) >= PATH_CAP) return 0;
    wcscpy_s(runner_dir, PATH_CAP, executable);
    parent_last = wcsrchr(runner_dir, L'\\');
    if (!parent_last) return 0;
    *parent_last = L'\0';
    if (wcslen(runner_dir) >= root_capacity) return 0;
    wcscpy_s(root, root_capacity, runner_dir);
    {
        wchar_t root_name[64];
        wchar_t parent_name[64];
        wchar_t grandparent_name[64];
        wchar_t parent_copy[PATH_CAP];
        wchar_t grandparent_copy[PATH_CAP];
        wcscpy_s(parent_copy, PATH_CAP, root);
        last = wcsrchr(parent_copy, L'\\');
        if (!last) return 0;
        *last = L'\0';
        wcscpy_s(grandparent_copy, PATH_CAP, parent_copy);
        last = wcsrchr(grandparent_copy, L'\\');
        if (!last) return 0;
        *last = L'\0';
        wcscpy_s(root_name, 64, wcsrchr(root, L'\\') ? wcsrchr(root, L'\\') + 1 : root);
        wcscpy_s(parent_name, 64, wcsrchr(parent_copy, L'\\') ? wcsrchr(parent_copy, L'\\') + 1 : parent_copy);
        wcscpy_s(grandparent_name, 64, wcsrchr(grandparent_copy, L'\\') ? wcsrchr(grandparent_copy, L'\\') + 1 : grandparent_copy);
        if (_wcsicmp(root_name, L"loader-v1") || _wcsicmp(parent_name, L"tests") || _wcsicmp(grandparent_name, L"lxmi")) return 0;
    }
    if (!append_path(target, target_capacity, root, L"target\\lxmi-loader-test-target.exe") ||
        !append_path(runtime, runtime_capacity, root, L"runtime") ||
        !append_path(results, results_capacity, root, L"results")) return 0;
    return 1;
}

static int launch_target(const wchar_t *mode, const wchar_t *nonce, const wchar_t *target_path,
                         const wchar_t *runtime_path, const wchar_t *results_path,
                         PROCESS_INFORMATION *process, RunnerResult *result) {
    wchar_t stdout_path[PATH_CAP];
    wchar_t stderr_path[PATH_CAP];
    wchar_t ready_path[PATH_CAP];
    wchar_t command_line[PATH_CAP];
    wchar_t target_directory[PATH_CAP];
    SECURITY_ATTRIBUTES security = { sizeof(SECURITY_ATTRIBUTES), NULL, TRUE };
    STARTUPINFOW startup;
    HANDLE stdout_file = INVALID_HANDLE_VALUE;
    HANDLE stderr_file = INVALID_HANDLE_VALUE;
    BOOL created;
    char ready[160];
    int written;
    wchar_t *last;

    if (!append_path(stdout_path, PATH_CAP, results_path, L"target.stdout.log") ||
        !append_path(stderr_path, PATH_CAP, results_path, L"target.stderr.log") ||
        !append_path(ready_path, PATH_CAP, results_path, L"target-ready.json")) return 0;
    stdout_file = CreateFileW(stdout_path, GENERIC_WRITE, FILE_SHARE_READ, &security, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    stderr_file = CreateFileW(stderr_path, GENERIC_WRITE, FILE_SHARE_READ, &security, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (stdout_file == INVALID_HANDLE_VALUE || stderr_file == INVALID_HANDLE_VALUE) goto failed;

    SetEnvironmentVariableW(L"LXMI_LOADER_TEST_NONCE", nonce);
    SetEnvironmentVariableW(L"LXMI_LOADER_TEST_PHASE", _wcsicmp(mode, L"baseline") == 0 ? L"baseline" : L"loader");
    ZeroMemory(&startup, sizeof(startup));
    startup.cb = sizeof(startup);
    startup.dwFlags = STARTF_USESTDHANDLES;
    startup.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
    startup.hStdOutput = stdout_file;
    startup.hStdError = stderr_file;
    if (wcslen(target_path) >= PATH_CAP) goto failed;
    wcscpy_s(target_directory, PATH_CAP, target_path);
    last = wcsrchr(target_directory, L'\\');
    if (!last) goto failed;
    *last = L'\0';
    if (_snwprintf_s(command_line, PATH_CAP, _TRUNCATE, L"\"%ls\"", target_path) < 0) goto failed;
    created = CreateProcessW(target_path, command_line, NULL, NULL, TRUE,
                             CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW,
                             NULL, target_directory, &startup, process);
    CloseHandle(stdout_file);
    CloseHandle(stderr_file);
    if (!created) goto failed_closed;
    result->target_started = TRUE;

    written = snprintf(ready, sizeof(ready), "{\"protocol\":1,\"ready\":true,\"nonce\":\"%ls\"}\n", nonce);
    if (written <= 0 || (size_t)written >= sizeof(ready) ||
        !wait_for_exact_file(ready_path, ready, 5000)) {
        result->diagnostic = "target_not_ready";
        TerminateProcess(process->hProcess, 97);
        WaitForSingleObject(process->hProcess, 1000);
        result->target_exit_code = 97;
        CloseHandle(process->hThread);
        CloseHandle(process->hProcess);
        return 0;
    }
    result->target_ready = TRUE;
    return 1;

failed:
    if (stdout_file != INVALID_HANDLE_VALUE) CloseHandle(stdout_file);
    if (stderr_file != INVALID_HANDLE_VALUE) CloseHandle(stderr_file);
failed_closed:
    (void)runtime_path;
    return 0;
}

int wmain(int argc, wchar_t **argv) {
    RunnerResult result;
    wchar_t root[PATH_CAP];
    wchar_t target_path[PATH_CAP];
    wchar_t runtime_path[PATH_CAP];
    wchar_t results_path[PATH_CAP];
    wchar_t loader_path[PATH_CAP];
    wchar_t dll_path[PATH_CAP];
    wchar_t missing_target[PATH_CAP];
    wchar_t missing_dll[PATH_CAP];
    wchar_t marker_path[PATH_CAP];
    wchar_t ready_path[PATH_CAP];
    wchar_t stop_path[PATH_CAP];
    wchar_t nonce[64];
    wchar_t target_nonce[64];
    wchar_t expected_target_name[64];
    HMODULE loader = NULL;
    UpstreamInject inject = NULL;
    FARPROC inject_export = NULL;
    PROCESS_INFORMATION process;
    const wchar_t *mode;
    DWORD nonce_count;
    DWORD target_waited;
    DWORD exit_code = 0;
    BOOL target_exit_ok = FALSE;
    int return_code = 0;

    ZeroMemory(&result, sizeof(result));
    ZeroMemory(&process, sizeof(process));
    result.inject_code = -1;
    result.target_exit_code = -1;
    strcpy_s(result.marker_nonce, sizeof(result.marker_nonce), "");
    if (argc != 3 || !derive_layout(root, PATH_CAP, target_path, PATH_CAP, runtime_path, PATH_CAP, results_path, PATH_CAP)) {
        result.mode = "invalid_request";
        result.diagnostic = "invalid_test_layout_or_arguments";
        emit_result(&result);
        return 20;
    }
    mode = argv[1];
    nonce_count = (DWORD)wcslen(argv[2]);
    if (nonce_count >= sizeof(nonce) / sizeof(nonce[0]) || !valid_nonce(argv[2])) {
        result.mode = "invalid_request";
        result.diagnostic = "invalid_nonce";
        emit_result(&result);
        return 20;
    }
    wcscpy_s(nonce, sizeof(nonce) / sizeof(nonce[0]), argv[2]);
    wcscpy_s(target_nonce, sizeof(target_nonce) / sizeof(target_nonce[0]), nonce);
    if (_wcsicmp(mode, L"wrong-nonce") == 0) {
        target_nonce[0] = target_nonce[0] == L'0' ? L'1' : L'0';
    }
    if (_wcsicmp(mode, L"baseline") == 0) result.mode = "baseline";
    else if (_wcsicmp(mode, L"positive") == 0) result.mode = "direct_inject";
    else if (_wcsicmp(mode, L"missing-target") == 0) result.mode = "missing_target";
    else if (_wcsicmp(mode, L"missing-dll") == 0) result.mode = "missing_dll";
    else if (_wcsicmp(mode, L"wrong-nonce") == 0) result.mode = "wrong_nonce";
    else { result.mode = "invalid_request"; result.diagnostic = "unsupported_mode"; emit_result(&result); return 20; }

    if (!append_path(loader_path, PATH_CAP, runtime_path, L"3dmloader.dll") ||
        !append_path(dll_path, PATH_CAP, runtime_path, L"lxmi-loader-test.dll") ||
        !append_path(marker_path, PATH_CAP, results_path, L"loader-result.json") ||
        !append_path(ready_path, PATH_CAP, results_path, L"target-ready.json") ||
        !append_path(stop_path, PATH_CAP, results_path, L"stop.json")) {
        result.diagnostic = "test_path_too_long";
        emit_result(&result);
        return 20;
    }
    DeleteFileW(marker_path);
    DeleteFileW(ready_path);
    DeleteFileW(stop_path);
    if (_wcsicmp(mode, L"baseline") != 0 && !safe_regular_file(loader_path)) {
        result.diagnostic = "upstream_loader_missing_or_unsafe";
        emit_result(&result);
        return 21;
    }
    if (_wcsicmp(mode, L"positive") == 0 && !safe_regular_file(dll_path)) {
        result.diagnostic = "test_dll_missing_or_unsafe";
        emit_result(&result);
        return 22;
    }

    wcscpy_s(expected_target_name, sizeof(expected_target_name) / sizeof(expected_target_name[0]), L"lxmi-loader-test-target.exe");
    if (_wcsicmp(mode, L"missing-target") == 0) {
        if (!append_path(missing_target, PATH_CAP, root, L"target\\lxmi-loader-test-target-missing.exe")) {
            result.diagnostic = "invalid_test_layout";
            emit_result(&result);
            return 20;
        }
        wcscpy_s(target_path, PATH_CAP, missing_target);
    } else if (!safe_regular_file(target_path)) {
        result.diagnostic = "test_target_missing_or_unsafe";
        emit_result(&result);
        return 21;
    }

    if (_wcsicmp(mode, L"missing-target") == 0) {
        wchar_t command_line[PATH_CAP];
        STARTUPINFOW startup;
        ZeroMemory(&startup, sizeof(startup));
        ZeroMemory(&process, sizeof(process));
        startup.cb = sizeof(startup);
        if (_snwprintf_s(command_line, PATH_CAP, _TRUNCATE, L"\"%ls\"", target_path) < 0 ||
            CreateProcessW(target_path, command_line, NULL, NULL, FALSE, CREATE_NO_WINDOW, NULL, NULL, &startup, &process)) {
            if (process.hProcess) { TerminateProcess(process.hProcess, 99); CloseHandle(process.hProcess); }
            if (process.hThread) CloseHandle(process.hThread);
            result.diagnostic = "wrong_target_unexpectedly_started";
            emit_result(&result);
            return 23;
        }
        result.diagnostic = "target_not_found";
        emit_result(&result);
        return 10;
    }

    if (!launch_target(mode, target_nonce, target_path, runtime_path, results_path, &process, &result)) {
        if (!result.diagnostic) result.diagnostic = "target_launch_failed";
        emit_result(&result);
        return 24;
    }
    if (_wcsicmp(mode, L"baseline") == 0) {
        target_exit_ok = wait_target(&process, 5000, &result.target_exit_code);
        result.diagnostic = target_exit_ok && result.target_exit_code == 0 && !safe_regular_file(marker_path) ? NULL : "baseline_failed";
        if (!target_exit_ok) return_code = 25;
        else if (result.diagnostic) return_code = 26;
        emit_result(&result);
        return return_code;
    }

    loader = LoadLibraryW(loader_path);
    if (!loader) {
        result.diagnostic = "upstream_loader_load_failed";
        write_stop_file(stop_path);
        target_exit_ok = wait_target(&process, 2000, &result.target_exit_code);
        if (!target_exit_ok) { TerminateProcess(process.hProcess, 98); WaitForSingleObject(process.hProcess, 1000); }
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        emit_result(&result);
        return 27;
    }
    result.loader_loaded = TRUE;
    inject_export = GetProcAddress(loader, "Inject");
    if (!inject_export || sizeof(inject) != sizeof(inject_export)) {
        result.diagnostic = "upstream_inject_export_missing";
        FreeLibrary(loader);
        write_stop_file(stop_path);
        target_exit_ok = wait_target(&process, 2000, &result.target_exit_code);
        if (!target_exit_ok) { TerminateProcess(process.hProcess, 98); WaitForSingleObject(process.hProcess, 1000); }
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        emit_result(&result);
        return 28;
    }
    memcpy(&inject, &inject_export, sizeof(inject));

    if (_wcsicmp(mode, L"missing-dll") == 0) {
        if (!append_path(missing_dll, PATH_CAP, runtime_path, L"lxmi-loader-test-missing.dll")) {
            result.diagnostic = "invalid_test_layout";
            return_code = 29;
        } else {
            result.inject_called = TRUE;
            result.inject_code = inject(process.dwProcessId, missing_dll, 5);
            result.diagnostic = result.inject_code == 110 ? "missing_test_dll" : "wrong_dll_case_unexpected_result";
            return_code = result.inject_code == 110 ? 11 : 30;
        }
        write_stop_file(stop_path);
    } else {
        result.inject_called = TRUE;
        result.inject_code = inject(process.dwProcessId, dll_path, 5);
        if (result.inject_code != 0) {
            result.diagnostic = "upstream_inject_failed";
            return_code = 31;
            write_stop_file(stop_path);
        } else {
            DWORD start = GetTickCount();
            char marker[TEXT_CAP];
            DWORD length = 0;
            BOOL marker_readable = FALSE;
            while (GetTickCount() - start < 5000) {
                if (read_small_file(marker_path, marker, sizeof(marker), &length)) {
                    marker_readable = TRUE;
                    break;
                }
                Sleep(25);
            }
            if (marker_readable) {
                inspect_marker(marker_path, &result);
                char expected[33];
                if (!WideCharToMultiByte(CP_UTF8, 0, nonce, -1, expected, sizeof(expected), NULL, NULL)) expected[0] = '\0';
                result.nonce_verified = result.marker_nonce[0] && strcmp(result.marker_nonce, expected) == 0;
                if (_wcsicmp(mode, L"wrong-nonce") == 0) {
                    result.diagnostic = result.nonce_verified ? "wrong_nonce_test_unexpectedly_passed" : "nonce_mismatch";
                    return_code = result.nonce_verified ? 38 : 12;
                } else {
                    result.diagnostic = result.marker_process_matches && result.nonce_verified ? NULL : "marker_identity_or_nonce_mismatch";
                    return_code = result.diagnostic ? 32 : 0;
                }
            } else {
                result.diagnostic = "dll_marker_timeout";
                return_code = 34;
                write_stop_file(stop_path);
            }
        }
    }

    if (loader) FreeLibrary(loader);
    if (!file_exists(stop_path) && !file_exists(marker_path)) write_stop_file(stop_path);
    target_waited = WaitForSingleObject(process.hProcess, 5000);
    if (target_waited == WAIT_TIMEOUT) {
        TerminateProcess(process.hProcess, 98);
        WaitForSingleObject(process.hProcess, 1000);
        result.target_exit_code = 98;
        result.diagnostic = result.diagnostic ? result.diagnostic : "target_exit_timeout";
        if (return_code == 0) return_code = 35;
    } else if (target_waited == WAIT_OBJECT_0 && GetExitCodeProcess(process.hProcess, &exit_code)) {
        result.target_exit_code = (int)exit_code;
        if (exit_code != 0 && return_code == 0) {
            result.diagnostic = "target_nonzero_exit";
            return_code = 36;
        }
    } else if (return_code == 0) {
        result.diagnostic = "target_wait_failed";
        return_code = 37;
    }
    if (_wcsicmp(mode, L"missing-dll") == 0 && result.inject_code == 110 && result.target_exit_code == 0) result.diagnostic = "missing_test_dll";
    if (_wcsicmp(mode, L"wrong-nonce") == 0 && result.marker_present && !result.nonce_verified && result.target_exit_code == 0) result.diagnostic = "nonce_mismatch";
    CloseHandle(process.hThread);
    CloseHandle(process.hProcess);
    emit_result(&result);
    return return_code;
}
