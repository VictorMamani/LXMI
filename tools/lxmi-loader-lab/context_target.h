/* Session-specific self identity; metadata only, no process enumeration or memory access. */
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
    if (!p || (wcscmp(p + 1, L"direct-inject-boundary") && wcscmp(p + 1, L"context-reentry-v1"))) return 0;
    *p = 0; p = wcsrchr(check, L'\\');
    if (!p || wcscmp(p + 1, L"tests")) return 0;
    *p = 0; p = wcsrchr(check, L'\\');
    if (!p || wcscmp(p + 1, L"lxmi")) return 0;
    return 1;
}

static unsigned long long context_time(FILETIME time) {
    return ((unsigned long long)time.dwHighDateTime << 32) | time.dwLowDateTime;
}
static void context_escape(const wchar_t *value, char *output, size_t cap) {
    char utf8[PATH_CAP * 4] = {0}; size_t n = 0;
    if (!WideCharToMultiByte(CP_UTF8, 0, value, -1, utf8, sizeof(utf8), NULL, NULL)) { output[0]=0; return; }
    for (const unsigned char *p=(unsigned char *)utf8; *p; ++p) {
        if (*p < 32 || n + 3 >= cap) { output[0]=0; return; }
        if (*p == '\\' || *p == '"') output[n++]='\\';
        output[n++]=(char)*p;
    }
    output[n]=0;
}
#ifndef CONTEXT_RUNNER
static int context_target(const wchar_t *nonce, BOOL reentry) {
    wchar_t root[PATH_CAP], ready[PATH_CAP], stop[PATH_CAP], marker[PATH_CAP], identity_path[PATH_CAP];
    wchar_t executable[PATH_CAP];
    DWORD length=GetModuleFileNameW(NULL, executable, PATH_CAP);
    if (!length || length>=PATH_CAP) return 20;
    wchar_t *base = wcsrchr(executable, L'\\');
    if (!base || wcscmp(base + 1, L"lxmi-loader-test-target.exe") || !context_layout(root, nonce)) return 20;
    swprintf(ready, PATH_CAP, L"%ls\\results\\target-ready", root);
    swprintf(identity_path, PATH_CAP, L"%ls\\results\\target-identity.json", root);
    swprintf(stop, PATH_CAP, L"%ls\\results\\stop", root);
    swprintf(marker, PATH_CAP, L"%ls\\results\\filesystem-marker", root);
    char nonce_utf8[33]={0}; for(int i=0;i<32;++i) nonce_utf8[i]=(char)nonce[i];
    char data[33]={0}; DWORD count=0;
    HANDLE f=CreateFileW(marker,GENERIC_READ,FILE_SHARE_READ,NULL,OPEN_EXISTING,0,NULL);
    BOOL readable=f!=INVALID_HANDLE_VALUE && ReadFile(f,data,32,&count,NULL) && count==32 && !strcmp(data,nonce_utf8);
    if(f!=INVALID_HANDLE_VALUE) CloseHandle(f);
    FILETIME creation,exit,kernel,user;
    if(!readable || !GetProcessTimes(GetCurrentProcess(),&creation,&exit,&kernel,&user)) return 21;
    HANDLE event=NULL;
    if(reentry) {
        wchar_t event_name[128];swprintf(event_name,128,L"Local\\LXMIReentryV1-%ls",nonce);
        event=CreateEventW(NULL,TRUE,FALSE,event_name);
        if(!event || GetLastError()==ERROR_ALREADY_EXISTS) {if(event)CloseHandle(event);return 24;}
    }
    char image[PATH_CAP * 8];context_escape(executable,image,sizeof(image));
    char json[PATH_CAP * 8 + 512];
    int size=snprintf(json,sizeof(json),"{\"protocol\":2,\"pid\":%lu,\"nonce\":\"%s\",\"process_image\":\"%s\",\"creation_time\":\"%llu\",\"ready\":true}\n",GetCurrentProcessId(),nonce_utf8,image,context_time(creation));
    if(!image[0] || size<=0 || (size_t)size>=sizeof(json)) return 22;
    f=CreateFileW(identity_path,GENERIC_WRITE,0,NULL,CREATE_NEW,0,NULL);
    if(f==INVALID_HANDLE_VALUE) return 22;
    BOOL written=WriteFile(f,json,(DWORD)size,&count,NULL) && count==(DWORD)size;
    FlushFileBuffers(f);CloseHandle(f);if(!written)return 22;
    f=CreateFileW(ready,GENERIC_WRITE,0,NULL,CREATE_NEW,0,NULL);
    if(f==INVALID_HANDLE_VALUE)return 22;
    written=WriteFile(f,nonce_utf8,32,&count,NULL) && count==32;FlushFileBuffers(f);CloseHandle(f);if(!written)return 22;
    DWORD start=GetTickCount();
    while(GetTickCount()-start<30000 && !exists(stop)) Sleep(25);
    printf("{\"protocol\":2,\"pid\":%lu,\"nonce\":\"%s\",\"process_image\":\"%s\",\"creation_time\":\"%llu\",\"ready\":true,\"path_readable\":true,\"handshake_sent\":false,\"wine_prefix\":",GetCurrentProcessId(),nonce_utf8,image,context_time(creation));
    BOOL received=event && WaitForSingleObject(event,0)==WAIT_OBJECT_0;
    if(event)CloseHandle(event);
    context_env("WINEPREFIX");printf(",\"compatdata\":");context_env("STEAM_COMPAT_DATA_PATH");
    if(reentry)printf(",\"event_received\":%s",received?"true":"false");
    puts("}");
    return 0;
}
#endif
