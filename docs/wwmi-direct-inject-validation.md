# WWMI Direct Inject validation

Validated on 2026-10-19. WWMI remains experimental because gameplay confirmation
covers one local Steam, Proton, and DXVK combination.

## Tested setup

| Component | Tested value |
|---|---|
| Steam game | Wuthering Waves, AppID 3513350 |
| Explicit compatibility tool | dwproton 11.0-14 |
| Container | Steam Linux Runtime 4 |
| Steam launch options | `PROTON_DXVK_SAREK=1 %command%` |
| Graphics argument supplied by LXMI | `-dx11` |
| Importer | Official WWMI package v1.1.0; internal version 1.10 |
| Shared libraries | Official XXMI Libraries 1.1.7 |

LXMI reads the per-game compatibility tool and its `require_tool_appid` from
Steam's files. A missing or unsupported explicit choice blocks launch with a
configuration message. ZZZ retains its launcher-first hook flow.

## Native launch evidence

The loader container starts before Steam is asked to launch WuWa, but waits before
starting Wine. Once Steam's game context appears, LXMI checks the observed Proton
and runtime when available and releases the loader into that wineserver. This
avoids Steam's `waitforexitandrun` waiting for a loader-owned wineserver.

The helper loads the importer's own Microsoft `d3dcompiler_47.dll` before
`d3d11.dll`. The native test recorded:

```jsonl
{"status":"compiler_inject_returned","code":0}
{"status":"inject_returned","code":0}
{"status":"game_observed","windows_pid":336,"mods_module_present":1,"compiler_module_present":1,"process_age_ms":1538}
{"status":"game_alive_after_load","observation_ms":15000}
```

This record came from the RabbitFX-only isolation run
`001792449107-81c7d12a0faae742`. Across reported tests, module observation occurred
about 1.3–1.9 seconds after game creation. The helper accepts a game process up to
60 seconds older than itself only in Direct Inject mode; LXMI checks the game is
closed before requesting a launch.

Module presence and the 15-second health check establish loading and early process
survival. They do not establish successful shader compilation, rendered mod
effects, sustained gameplay, or account safety.

## Gameplay confirmation and DXVK limitation

The user subsequently confirmed entry into gameplay with Hiyuki, Hsin, and RabbitFX,
including RabbitFX's remove-censor option. With the selected dwproton version,
default DXVK 3.1.1 crashed near 75% loading. Enabling `PROTON_DXVK_SAREK=1` in Steam
selected DXVK Sarek and allowed gameplay. This is a reported compatibility result;
it does not establish the internal cause of the crash or qualify other versions.

Keep the Sarek launch option for the tested setup. LXMI shows a setup notice but
does not set this option, inspect its value, or certify the selected DXVK at
runtime. No anti-cheat bypass is implemented.

## Mod issues outside this change

- The original Hsin `mod.ini` refers to an undeclared `$draw_19`. A corrected local
  copy still needs to be imported separately if the user chooses to use it.
- Moonover Fox remains excluded from the final reported working profile because
  its original INI has unclosed conditional blocks.
- Imported mod contents are verified by hash. Fix source files and import them
  again rather than editing the private library or an immutable runtime snapshot.

Temporary native launch harnesses and user mod files are not part of this change.
Permanent opt-in tests for Steam context inspection and container staging remain
ignored by default; ordinary automated checks do not launch a game.

## Commit validation

- Rust formatting and workspace Clippy with warnings denied: passed.
- Workspace tests: 435 passed; 15 opt-in tests ignored.
- Frontend regressions: 6 passed, including the different WWMI and ZZZ step order.
- TypeScript, ESLint, Prettier, and production frontend build: passed.
- Browser demo: Steam setup notice and consent inspected at 1366×768 and 1024×640,
  without dialog overflow. This copy check did not launch a game or replace the
  native and user-reported evidence above.
