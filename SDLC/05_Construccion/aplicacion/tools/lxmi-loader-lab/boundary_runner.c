/* Exact fixture PID/image/creation-time preflight. No PID argv input or enumeration. */
#include <windows.h>
#include <stdio.h>
#include <wchar.h>
#include <string.h>
#define PATH_CAP 4096
#ifndef LXMI_REENTRY_ONLY
typedef int (__cdecl *UpstreamInject)(DWORD, LPCWSTR, int);
#endif
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

    wchar_t root[PATH_CAP], dll[PATH_CAP], loader_path[PATH_CAP], marker[PATH_CAP], request_path[PATH_CAP];
    if(argc!=2 || !context_layout(root,argv[1]))return 20;
    swprintf(dll,PATH_CAP,L"%ls\\runtime\\lxmi-loader-test.dll",root);
    swprintf(loader_path,PATH_CAP,L"%ls\\runtime\\3dmloader.dll",root);
    swprintf(marker,PATH_CAP,L"%ls\\results\\filesystem-marker",root);
    swprintf(request_path,PATH_CAP,L"%ls\\results\\identity-request.bin",root);
    char nonce[33]={0};for(int i=0;i<32;++i)nonce[i]=(char)argv[1][i];
    char data[33]={0};DWORD count=0;
    HANDLE f=CreateFileW(marker,GENERIC_READ,FILE_SHARE_READ,NULL,OPEN_EXISTING,0,NULL);
    BOOL readable=f!=INVALID_HANDLE_VALUE && ReadFile(f,data,32,&count,NULL) && count==32 && !strcmp(data,nonce);
    if(f!=INVALID_HANDLE_VALUE)CloseHandle(f);
    /* Controller-generated request, fixed filename/layout. No PID argv input. */
    DWORD protocol=0,expected_pid=0,image_length=0;
    unsigned long long expected_time=0;char expected_nonce[32];wchar_t expected_image[PATH_CAP]={0},fixed_image[PATH_CAP];
    f=CreateFileW(request_path,GENERIC_READ,FILE_SHARE_READ,NULL,OPEN_EXISTING,0,NULL);
    if(f==INVALID_HANDLE_VALUE)return 23;
#define READ_FIELD(value) (ReadFile(f,&value,sizeof(value),&count,NULL) && count==sizeof(value))
    BOOL request_ok=READ_FIELD(protocol) && READ_FIELD(expected_pid) && READ_FIELD(expected_time) && READ_FIELD(expected_nonce) && READ_FIELD(image_length);
    request_ok=request_ok && protocol==2 && expected_pid && expected_time && image_length>0 && image_length<PATH_CAP && !memcmp(expected_nonce,nonce,32);
    if(request_ok)request_ok=ReadFile(f,expected_image,image_length*2,&count,NULL) && count==image_length*2;
    BYTE trailing;DWORD tail=0;
    if(request_ok)request_ok=ReadFile(f,&trailing,1,&tail,NULL) && tail==0;
    CloseHandle(f);
    swprintf(fixed_image,PATH_CAP,L"%ls\\target\\lxmi-loader-test-target.exe",root);
    if(!request_ok || !readable || wcslen(expected_image)!=image_length || _wcsicmp(expected_image,fixed_image))return 23;
    /* Exactly one PID query, limited metadata rights only. No enumeration,
       debug privilege, memory rights or thread-creation rights. */
    HANDLE owned=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,FALSE,expected_pid);
    DWORD error=0,observed_pid=0;unsigned long long observed_time=0;wchar_t actual[PATH_CAP]={0};
    const char *identity="unknown";
    if(!owned) {
        error=GetLastError();identity=error==ERROR_ACCESS_DENIED?"access_denied":error==ERROR_INVALID_PARAMETER?"pid_not_visible":"unknown";
    } else {
        DWORD capacity=PATH_CAP,exit_code=0;FILETIME creation,exit,kernel,user;
        observed_pid=GetProcessId(owned);
        if(!observed_pid || !QueryFullProcessImageNameW(owned,0,actual,&capacity) || !GetProcessTimes(owned,&creation,&exit,&kernel,&user) || !GetExitCodeProcess(owned,&exit_code)) {
            error=GetLastError();identity=error==ERROR_ACCESS_DENIED?"access_denied":"unknown";
        } else {
            observed_time=context_time(creation);
            if(exit_code!=STILL_ACTIVE)identity="process_exited";
            else if(observed_pid!=expected_pid || observed_pid==GetCurrentProcessId())identity="pid_collision";
            else if(_wcsicmp(actual,expected_image))identity="image_mismatch";
            else if(observed_time!=expected_time)identity="creation_time_mismatch";
            else identity="verified";
        }
    }
    BOOL bound=!strcmp(identity,"verified"),called=FALSE;int code=-1;
    /* The retained query handle prevents reuse of the identified process
       object while upstream resolves its PID. Recheck liveness before Inject. */
    if(bound) {
        DWORD exit_code=0;
        if(!GetExitCodeProcess(owned,&exit_code)) {error=GetLastError();identity="unknown";bound=FALSE;}
        else if(exit_code!=STILL_ACTIVE) {identity="process_exited";bound=FALSE;}
    }
    #ifndef LXMI_REENTRY_ONLY
    if(bound) {
        HMODULE loader=LoadLibraryW(loader_path);
        if(loader) {
            FARPROC symbol=GetProcAddress(loader,"Inject");UpstreamInject inject=NULL;
            if(symbol && sizeof(symbol)==sizeof(inject)) {memcpy(&inject,&symbol,sizeof(inject));called=TRUE;code=inject(expected_pid,dll,5);}
            else error=GetLastError();
            FreeLibrary(loader);
        } else error=GetLastError();
    }
    #endif
    #ifdef LXMI_REENTRY_ONLY
    wchar_t event_name[128];swprintf(event_name,128,L"Local\\LXMIReentryV1-%ls",argv[1]);
    HANDLE event=OpenEventW(EVENT_MODIFY_STATE,FALSE,event_name);
    DWORD event_error=event?0:GetLastError();
    BOOL event_visible=event && SetEvent(event);
    if(event)CloseHandle(event);
    #endif
    if(owned)CloseHandle(owned);
    char expected_json[PATH_CAP*8],actual_json[PATH_CAP*8];context_escape(expected_image,expected_json,sizeof(expected_json));context_escape(actual,actual_json,sizeof(actual_json));
    printf("{\"protocol\":2,\"pid\":%lu,\"target_pid\":%lu,\"nonce\":\"%s\",\"path_readable\":true,\"identity_bound\":%s,\"inject_called\":%s,\"inject_code\":%d,\"windows_error\":%lu,\"target_identity\":{\"expected_windows_pid\":%lu,\"observed_windows_pid\":%lu,\"expected_image\":\"%s\",\"observed_image\":\"%s\",\"expected_creation_time\":\"%llu\",\"observed_creation_time\":\"%llu\",\"nonce\":\"%s\",\"result\":\"%s\"},\"wine_prefix\":",GetCurrentProcessId(),expected_pid,nonce,bound?"true":"false",called?"true":"false",code,error,expected_pid,observed_pid,expected_json,actual_json,expected_time,observed_time,nonce,identity);
    context_env("WINEPREFIX");printf(",\"compatdata\":");context_env("STEAM_COMPAT_DATA_PATH");
    #ifdef LXMI_REENTRY_ONLY
    printf(",\"named_event_visible\":%s,\"event_error\":%lu",event_visible?"true":"false",event_error);
    #endif
    puts("}");
    return 0;
}
