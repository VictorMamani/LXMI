# LXMI Loader Lab (development)

These tools test the official `3dmloader.dll` `Inject` export against a single
Windows process created by LXMI. They do not accept arbitrary PIDs, process names,
or DLL paths. The runner derives paths from its own location under
`lxmi/tests/loader-v1/`.

The target is `lxmi-loader-test-target.exe`; the harmless DLL is
`lxmi-loader-test.dll`. The DLL writes a JSON marker to the laboratory results
directory only when its own process name exactly matches the LXMI target.

`3dmloader.dll` is not copied into this repository. The backend obtains it from an
XXMI Libraries package verified by the store as an official release, then copies
it into the private laboratory directory.

## Build

Ubuntu requirements: `gcc-mingw-w64-x86-64` and `mingw-w64-x86-64-dev`. Nothing is
installed automatically.

From the repository root:

```bash
bash tools/lxmi-loader-lab/build.sh
```

Output is written to `target/x86_64-pc-windows-gnu/release/lxmi-loader-lab/` using
the host MinGW compiler. The files are validated and copied to LXMI XDG storage
when you choose **Prepare** in the **Loader Lab** section of the UI.

## Restrictions

- `direct_inject` calls the upstream export with the PID returned by its child
  `CreateProcessW`; it does not search for processes.
- `hook` is not tested: upstream installs a global window hook.
- The runner starts only the fixed test executable; there is no path argument.
- The DLL checks its process name and a 128-bit nonce.
- Do not use with ZZZ, Steam, HoYoPlay, or another process.
- This directory contains only LXMI test code; it contains no upstream source code
  and redistributes no upstream binaries.
