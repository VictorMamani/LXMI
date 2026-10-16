/* Hook isolation fixture owned by LXMI. Never opens or launches a game process.
 *
 * The same PE is copied under three basenames; the role must match its name:
 *   lxmi-hook-loader.exe    controller: official HookLibrary/UnhookLibrary,
 *                           starts only its own sibling fixtures.
 *   lxmi-hook-target.exe    process named by [Loader] target.
 *   lxmi-hook-bystander.exe GUI process outside target/loader (HoYoPlay stand-in).
 * Images live in <base>\bin and the runtime in <base>\ZZMI: upstream accepts
 * any process whose image directory is contained in the DLL directory, so
 * the images must not sit in an ancestor of ZZMI. Paths derive from the image
 * location; argv carries no path or PID.
 */
#define WIN32_LEAN_AND_MEAN
#define COBJMACROS
#include <windows.h>
#include <psapi.h>
#include <d3d11.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

typedef int (*HookLibraryFn)(LPCWSTR, HHOOK *, HANDLE *);
typedef int (*UnhookLibraryFn)(HHOOK *, HANDLE *);
typedef HRESULT (WINAPI *CreateDeviceSwapChain)(IDXGIAdapter *, D3D_DRIVER_TYPE,
    HMODULE, UINT, const D3D_FEATURE_LEVEL *, UINT, UINT,
    const DXGI_SWAP_CHAIN_DESC *, IDXGISwapChain **, ID3D11Device **,
    D3D_FEATURE_LEVEL *, ID3D11DeviceContext **);

static wchar_t bin[4096], base[4096], xxmi[4096], role[64];

static void emit(const char *stage, long value) {
    printf("{\"role\":\"%ls\",\"pid\":%lu,\"stage\":\"%s\",\"value\":%ld}\n",
        role, (unsigned long)GetCurrentProcessId(), stage, value);
    fflush(stdout);
}

static void emit_module(const char *stage, const void *address) {
    HMODULE module = NULL;
    wchar_t path[4096] = L"";
    const wchar_t *kind = L"none";
    if (address && GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
            GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, (LPCWSTR)address, &module) &&
            GetModuleFileNameW(module, path, 4096)) {
        const wchar_t *base = wcsrchr(path, L'\\');
        kind = !_wcsicmp(path, xxmi) ? L"xxmi" : (base ? base + 1 : path);
    }
    printf("{\"role\":\"%ls\",\"pid\":%lu,\"stage\":\"%s\",\"module\":\"%ls\"}\n",
        role, (unsigned long)GetCurrentProcessId(), stage, kind);
    fflush(stdout);
}

/* Exact-path membership in this process; basename alone is not evidence. */
static int xxmi_mapped(void) {
    HMODULE modules[1024];
    DWORD needed = 0;
    if (!EnumProcessModules(GetCurrentProcess(), modules, sizeof(modules), &needed)) return -1;
    DWORD count = needed / sizeof(HMODULE);
    if (count > 1024) count = 1024;
    for (DWORD i = 0; i < count; i++) {
        wchar_t path[4096];
        if (GetModuleFileNameW(modules[i], path, 4096) && !_wcsicmp(path, xxmi)) return 1;
    }
    return 0;
}

static void pump(DWORD milliseconds) {
    DWORD start = GetTickCount();
    MSG msg;
    while (GetTickCount() - start < milliseconds) {
        while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(20);
    }
}

static HWND hidden_window(const wchar_t *name) {
    WNDCLASSW cls;
    memset(&cls, 0, sizeof(cls));
    cls.lpfnWndProc = DefWindowProcW;
    cls.hInstance = GetModuleHandleW(NULL);
    cls.lpszClassName = name;
    if (!RegisterClassW(&cls)) return NULL;
    /* Never shown: CreateWindowEx alone produces the HCBT_CREATEWND event. */
    return CreateWindowExW(0, name, name, WS_OVERLAPPEDWINDOW, 0, 0, 64, 64,
        NULL, NULL, cls.hInstance, NULL);
}

static int bystander(void) {
    emit("xxmi_mapped_before_window", xxmi_mapped());
    HWND window = hidden_window(L"LXMIHookBystander");
    if (!window) return 40;
    pump(1500);
    emit("xxmi_mapped_after_window", xxmi_mapped());
    DestroyWindow(window);
    emit("bystander_completed", 0);
    return 0;
}

/* early: system d3d11 resolved before the window, as when a game imports it;
 * late: d3d11 resolved after the window by basename. */
static int target(int early) {
    emit("xxmi_mapped_at_start", xxmi_mapped());
    HMODULE d3d11 = NULL;
    FARPROC symbol = NULL;
    if (early) {
        d3d11 = LoadLibraryW(L"d3d11.dll");
        if (!d3d11 || !(symbol = GetProcAddress(d3d11, "D3D11CreateDeviceAndSwapChain"))) return 41;
        emit_module("d3d11_resolved_before_window", d3d11);
    }
    HWND window = hidden_window(L"LXMIHookTarget");
    if (!window) return 42;
    pump(500);
    emit("xxmi_mapped_after_window", xxmi_mapped());
    if (!early) {
        d3d11 = LoadLibraryW(L"d3d11.dll");
        if (!d3d11 || !(symbol = GetProcAddress(d3d11, "D3D11CreateDeviceAndSwapChain"))) return 41;
        emit_module("d3d11_resolved_after_window", d3d11);
    }
    CreateDeviceSwapChain create = NULL;
    if (sizeof(symbol) != sizeof(create)) return 43;
    memcpy(&create, &symbol, sizeof(create));
    DXGI_SWAP_CHAIN_DESC desc;
    memset(&desc, 0, sizeof(desc));
    desc.BufferDesc.Width = 64;
    desc.BufferDesc.Height = 64;
    desc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.SampleDesc.Count = 1;
    desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    desc.BufferCount = 1;
    desc.OutputWindow = window;
    desc.Windowed = TRUE;
    desc.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;
    IDXGISwapChain *swap = NULL;
    ID3D11Device *device = NULL;
    ID3D11DeviceContext *context = NULL;
    D3D_FEATURE_LEVEL level;
    emit("xxmi_mapped_before_device", xxmi_mapped());
    HRESULT hr = create(NULL, D3D_DRIVER_TYPE_HARDWARE, NULL, 0, NULL, 0,
        D3D11_SDK_VERSION, &desc, &swap, &device, &level, &context);
    emit("create_device_hresult", (long)hr);
    if (FAILED(hr)) return 44;
    /* Where the device's interface and one of its methods live: a wrapper or
     * vtable hook points into the XXMI module, an untouched device does not. */
    void **vtable = *(void ***)device;
    emit_module("device_vtable_module", vtable);
    emit_module("device_create_buffer_module", vtable[3]);
    hr = IDXGISwapChain_Present(swap, 0, 0);
    emit("present_hresult", (long)hr);
    if (context) ID3D11DeviceContext_Release(context);
    if (swap) IDXGISwapChain_Release(swap);
    ID3D11Device_Release(device);
    DestroyWindow(window);
    emit("target_completed", 0);
    return FAILED(hr) ? 45 : 0;
}

static long run_sibling(const wchar_t *exe, const wchar_t *args, const char *label) {
    wchar_t path[4096], line[8192];
    if (swprintf(path, 4096, L"%ls\\%ls", bin, exe) < 0) return -1;
    if (swprintf(line, 8192, L"\"%ls\" %ls", path, args) < 0) return -1;
    STARTUPINFOW si;
    PROCESS_INFORMATION pi;
    memset(&si, 0, sizeof(si));
    si.cb = sizeof(si);
    memset(&pi, 0, sizeof(pi));
    if (!CreateProcessW(path, line, NULL, NULL, FALSE, 0, NULL, bin, &si, &pi)) {
        emit(label, -(long)GetLastError());
        return -1;
    }
    DWORD wait = WaitForSingleObject(pi.hProcess, 60000);
    DWORD code = 0;
    if (wait != WAIT_OBJECT_0) {
        /* Our own child only; never search or signal other processes. */
        TerminateProcess(pi.hProcess, 99);
        WaitForSingleObject(pi.hProcess, 5000);
        code = 0xFFFF;
    } else if (!GetExitCodeProcess(pi.hProcess, &code)) {
        code = 0xFFFE;
    }
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    emit(label, (long)code);
    /* Each XXMI attach truncates d3d11_log.txt: keep one copy per child. */
    wchar_t log[4096], kept[4096];
    if (swprintf(log, 4096, L"%ls\\ZZMI\\d3d11_log.txt", base) >= 0 &&
            swprintf(kept, 4096, L"%ls\\logs\\%hs.d3d11_log.txt", base, label) >= 0)
        emit("log_kept", CopyFileW(log, kept, TRUE) ? 1 : -(long)GetLastError());
    return (long)code;
}

static int wait_for_go(void) {
    wchar_t go[4096];
    if (swprintf(go, 4096, L"%ls\\go", base) < 0) return 0;
    for (int i = 0; i < 1800; i++) {
        if (GetFileAttributesW(go) != INVALID_FILE_ATTRIBUTES) return 1;
        Sleep(100);
    }
    return 0;
}

static int controller(void) {
    wchar_t loader_path[4096];
    if (swprintf(loader_path, 4096, L"%ls\\3dmloader.dll", base) < 0) return 30;
    HMODULE loader = LoadLibraryW(loader_path);
    if (!loader) { emit("loader_load_failed", (long)GetLastError()); return 31; }
    HookLibraryFn hook_library = (HookLibraryFn)(void *)GetProcAddress(loader, "HookLibrary");
    UnhookLibraryFn unhook_library = (UnhookLibraryFn)(void *)GetProcAddress(loader, "UnhookLibrary");
    if (!hook_library || !unhook_library) { emit("loader_exports_missing", 0); return 32; }
    HHOOK hook = NULL;
    HANDLE mutex = NULL;
    int code = hook_library(xxmi, &hook, &mutex);
    emit("hook_library_returned", code);
    if (code != 0) return 33;
    emit("controller_xxmi_mapped", xxmi_mapped());
    emit("hook_installed", 0);
    if (!wait_for_go()) {
        emit("go_timeout", 0);
        emit("unhook_library_returned", unhook_library(&hook, &mutex));
        return 34;
    }
    run_sibling(L"lxmi-hook-bystander.exe", L"bystander", "bystander_exit");
    run_sibling(L"lxmi-hook-target.exe", L"target-early", "target_early_exit");
    run_sibling(L"lxmi-hook-target.exe", L"target-late", "target_late_exit");
    /* Repeat both orders once to tell a reproducible result from a one-off. */
    run_sibling(L"lxmi-hook-target.exe", L"target-early", "target_early_repeat_exit");
    run_sibling(L"lxmi-hook-target.exe", L"target-late", "target_late_repeat_exit");
    code = unhook_library(&hook, &mutex);
    emit("unhook_library_returned", code);
    run_sibling(L"lxmi-hook-target.exe", L"target-early", "target_after_unhook_exit");
    emit("controller_completed", 0);
    return code == 0 ? 0 : 35;
}

int wmain(int argc, wchar_t **argv) {
    if (argc != 2 || wcslen(argv[1]) >= 64) return 20;
    wcscpy(role, argv[1]);
    DWORD n = GetModuleFileNameW(NULL, bin, 4096);
    if (!n || n >= 4096 || !wcsrchr(bin, L'\\')) return 21;
    const wchar_t *image = wcsrchr(bin, L'\\') + 1;
    const wchar_t *expected =
        !wcscmp(role, L"controller") ? L"lxmi-hook-loader.exe" :
        !wcscmp(role, L"bystander") ? L"lxmi-hook-bystander.exe" :
        (!wcscmp(role, L"target-early") || !wcscmp(role, L"target-late")) ? L"lxmi-hook-target.exe" : NULL;
    if (!expected || _wcsicmp(image, expected)) return 22;
    *wcsrchr(bin, L'\\') = 0;
    wcscpy(base, bin);
    if (!wcsrchr(base, L'\\') || _wcsicmp(wcsrchr(base, L'\\') + 1, L"bin")) return 23;
    *wcsrchr(base, L'\\') = 0;
    if (swprintf(xxmi, 4096, L"%ls\\ZZMI\\d3d11.dll", base) < 0) return 23;
    emit("started", 0);
    if (!wcscmp(role, L"controller")) return controller();
    if (!wcscmp(role, L"bystander")) return bystander();
    return target(!wcscmp(role, L"target-early"));
}
