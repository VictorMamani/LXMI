/* Diagnostic process owned by LXMI. Never opens or launches a game process. */
#define WIN32_LEAN_AND_MEAN
#define COBJMACROS
#include <windows.h>
#include <d3d11.h>
#include <stdio.h>
#include <wchar.h>
#include <string.h>

typedef HRESULT (WINAPI *CreateDeviceSwapChain)(IDXGIAdapter *, D3D_DRIVER_TYPE,
    HMODULE, UINT, const D3D_FEATURE_LEVEL *, UINT, UINT,
    const DXGI_SWAP_CHAIN_DESC *, IDXGISwapChain **, ID3D11Device **,
    D3D_FEATURE_LEVEL *, ID3D11DeviceContext **);
typedef HRESULT (WINAPI *CreateFactory)(REFIID, void **);

static void stage(const char *name, HRESULT hr) {
    printf("{\"stage\":\"%s\",\"hresult\":\"0x%08lx\"}\n", name, (unsigned long)hr);
    fflush(stdout);
}

int wmain(int argc, wchar_t **argv) {
    if (argc != 2 || (wcscmp(argv[1], L"native") && wcscmp(argv[1], L"xxmi") &&
        wcscmp(argv[1], L"hooked") && wcscmp(argv[1], L"hooked-device"))) return 20;
    wchar_t root[4096], dll[4096];
    DWORD n = GetModuleFileNameW(NULL, root, 4096);
    if (!n || n >= 4096 || !wcsrchr(root, L'\\')) return 20;
    *wcsrchr(root, L'\\') = 0;
    if (swprintf(dll, 4096, L"%ls\\ZZMI\\lxmi-xxmi.dll", root) < 0) return 20;
    stage("fixture_started", S_OK);
    HMODULE dxgi = LoadLibraryW(L"dxgi.dll");
    HMODULE native = LoadLibraryW(L"d3d11.dll");
    if (!dxgi || !native) { stage("native_libraries_failed", HRESULT_FROM_WIN32(GetLastError())); return 21; }
    HMODULE selected = native;
    if (wcscmp(argv[1], L"native")) {
        stage("before_xxmi_load", S_OK);
        selected = LoadLibraryExW(dll, NULL, LOAD_WITH_ALTERED_SEARCH_PATH);
        if (!selected) { stage("xxmi_load_failed", HRESULT_FROM_WIN32(GetLastError())); return 22; }
        wchar_t actual[4096];
        if (!GetModuleFileNameW(selected, actual, 4096) || _wcsicmp(actual, dll) || selected == native) {
            stage("xxmi_module_identity_failed", E_FAIL);
            return 28;
        }
        stage("xxmi_module_identity_verified", S_OK);
        stage("xxmi_loaded", S_OK);
    }
    HMODULE entry_module = !wcscmp(argv[1], L"xxmi") ? selected : native;
    FARPROC symbol = GetProcAddress(entry_module, "D3D11CreateDeviceAndSwapChain");
    CreateDeviceSwapChain create = NULL;
    if (!symbol || sizeof(symbol) != sizeof(create)) return 23;
    memcpy(&create, &symbol, sizeof(create));
    WNDCLASSW cls; memset(&cls, 0, sizeof(cls));
    cls.lpfnWndProc = DefWindowProcW;
    cls.hInstance = GetModuleHandleW(NULL);
    cls.lpszClassName = L"LXMIGraphicsFixture";
    if (!RegisterClassW(&cls)) return 24;
    /* Hidden window: one swapchain/present, no desktop input or game interaction. */
    HWND window = CreateWindowExW(0, cls.lpszClassName, L"LXMI graphics diagnostic",
        WS_OVERLAPPEDWINDOW, 0, 0, 64, 64, NULL, NULL, cls.hInstance, NULL);
    if (!window) return 25;
    DXGI_SWAP_CHAIN_DESC desc; memset(&desc, 0, sizeof(desc));
    desc.BufferDesc.Width = 64; desc.BufferDesc.Height = 64;
    desc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.SampleDesc.Count = 1;
    desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    desc.BufferCount = 1; desc.OutputWindow = window;
    desc.Windowed = TRUE; desc.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;
    IDXGISwapChain *swap = NULL; ID3D11Device *device = NULL;
    ID3D11DeviceContext *context = NULL; D3D_FEATURE_LEVEL level;
    stage("before_create_device", S_OK);
    HRESULT hr;
    if (!wcscmp(argv[1], L"hooked-device")) {
        PFN_D3D11_CREATE_DEVICE create_device = NULL;
        symbol = GetProcAddress(native, "D3D11CreateDevice");
        if (!symbol || sizeof(symbol) != sizeof(create_device)) return 29;
        memcpy(&create_device, &symbol, sizeof(create_device));
        hr = create_device(NULL, D3D_DRIVER_TYPE_HARDWARE, NULL, 0, NULL, 0,
            D3D11_SDK_VERSION, &device, &level, &context);
        stage("after_create_device_separate", hr);
        if (FAILED(hr)) return 26;
        CreateFactory factory_create = NULL;
        symbol = GetProcAddress(dxgi, "CreateDXGIFactory");
        if (!symbol || sizeof(symbol) != sizeof(factory_create)) return 30;
        memcpy(&factory_create, &symbol, sizeof(factory_create));
        IDXGIFactory *factory = NULL;
        hr = factory_create(&IID_IDXGIFactory, (void **)&factory);
        stage("after_create_factory", hr);
        if (FAILED(hr)) return 31;
        hr = IDXGIFactory_CreateSwapChain(factory, (IUnknown *)device, &desc, &swap);
        IDXGIFactory_Release(factory);
        stage("after_create_swapchain_separate", hr);
    } else {
        hr = create(NULL, D3D_DRIVER_TYPE_HARDWARE, NULL, 0, NULL, 0,
            D3D11_SDK_VERSION, &desc, &swap, &device, &level, &context);
    }
    stage("after_create_device", hr);
    if (FAILED(hr)) return 26;
    stage("before_present", S_OK);
    hr = IDXGISwapChain_Present(swap, 0, 0);
    stage("after_present", hr);
    if (context) ID3D11DeviceContext_Release(context);
    if (swap) IDXGISwapChain_Release(swap);
    if (device) ID3D11Device_Release(device);
    DestroyWindow(window);
    stage("fixture_completed", hr);
    /* Let process teardown unload modules; do not remove hooks while code runs. */
    return FAILED(hr) ? 27 : 0;
}
