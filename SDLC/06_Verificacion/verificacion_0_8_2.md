# LXMI 0.8.2 — Verificación de evidencia Steam / Proton

**Fecha:** 2026-09-30  
**Alcance:** inspección read-only, modelo de evidencia y UI de readiness. No se ejecutaron Steam, Proton, Wine, wineserver, ZZZ, XXMI Launcher ni loaders.

## Evidencia local

- **LOCAL VERIFIED:** `/home/university/.local/share/Steam/config/config.vdf` contiene `InstallConfigStore/Software/Valve/Steam/CompatToolMapping`, con objeto vacío. En este archivo no existe mapping para AppID `4162040` ni entrada global `0`.
- **LOCAL VERIFIED:** se buscaron referencias a `CompatToolMapping` en los `userdata/*/config/localconfig.vdf` existentes; no se encontraron. El servicio LXMI no interpreta esos archivos porque no hay una semántica de selección efectiva documentada allí en esta revisión.
- **LOCAL VERIFIED:** el manifest `/home/university/.local/share/Steam/steamapps/appmanifest_4162040.acf` contiene AppID `4162040`, nombre Zenless Zone Zero e `installdir` `Zenless Zone Zero`.
- **LOCAL VERIFIED:** existe `ZenlessZoneZero.exe` en el path esperado derivado del manifest.
- **LOCAL VERIFIED:** existen `steamapps/compatdata/4162040/` y `pfx/`. Ambos son observaciones/candidatos filesystem; no se abrió ni modificó el prefix.
- **LOCAL VERIFIED:** `Proton - Experimental` tiene entrypoint `proton`, `toolmanifest.vdf` con layer `proton` y versión raw `experimental-11.0-20260924-x86_64`. Es una herramienta instalada detectada, no una selección para el AppID.
- **LOCAL VERIFIED:** también existe el directorio `Proton Hotfix`, pero esta revisión no lo presenta como runtime válido sin metadata estructural que lo valide.

## Implementación y fixtures

- `lxmi-steam` reutiliza el parser Valve KeyValues y lee `config/config.vdf` con límite de 2 MiB. Rechaza symlinks en `config/` y en el archivo; reporta missing, invalid, unreadable, unsafe o too-large como estados distintos.
- **FIXTURE:** pruebas separan mapping por AppID `4162040`, mapping global `0`, sólo global, ausencia e input malformado. Un mapping global no se convierte en override del juego.
- **FIXTURE:** readiness recibe mapping explícito más varios Proton candidates; `selected_game_runtime` y el contexto efectivo permanecen `Unknown`.
- **FIXTURE:** compatdata/prefix ausentes se conservan independientemente del estado de instalación y no se confunden con selección efectiva.
- El panel ZZZ expone root/biblioteca/manifest/config, mappings, tools, rutas y relaciones desconocidas bajo detalles expandibles.

## Interpretación y límites

- **INFERRED:** manifest y AppID relacionan ZZZ con la biblioteca Steam encontrada; la ruta `compatdata/4162040/pfx` coincide con el layout esperado por AppID.
- **UPSTREAM VERIFIED:** la FAQ de Proton describe el prefix por juego en `steamapps/compatdata/<APPID>/pfx` ([snapshot de Valve Proton FAQ](https://github.com/ValveSoftware/Proton/wiki/Proton-FAQ/9a1684b9b948a44c8b7fda0c32d2fa50f8956bb4)). Esta convención no determina runtime efectivo ni salud del prefix.
- **UPSTREAM SOURCE, referencia móvil:** el wrapper público `proton_11.0` lee `STEAM_COMPAT_DATA_PATH` y compone `pfx` debajo de esa ruta ([código de Proton](https://github.com/ValveSoftware/Proton/blob/proton_11.0/proton)); el enlace apunta a una referencia de rama, no a un commit fijado. No prueba qué configuración ejecutó este juego.
- **UPSTREAM-HOSTED COMMUNITY REPORT:** el issue [ValveSoftware/steam-for-linux #10184](https://github.com/ValveSoftware/steam-for-linux/issues/10184) describe `CompatToolMapping` como mapping por AppID más entrada global `0`. Es un reporte de usuario en el tracker, no una especificación de Valve.
- **UNKNOWN:** qué compatibility tool usaría Steam efectivamente para ZZZ; `WINEPREFIX` efectivo; wineserver y desktop/session activos; si loader y juego deben compartir prefix, wineserver o build Proton; proceso-contexto futuro de ZZZ/loader.
- **ARCHITECTURAL DECISION:** mapping configurado, candidate instalado, compatdata y prefix candidato son campos separados del contexto efectivo. Sin proceso observado, runtime/prefix/wineserver efectivos siguen `Unknown`.

Readiness continúa `NeedsEvidence`; `LoaderStrategyDecision` permanece `NoDecision`; Controlled Game Experiment Gate no es elegible y no contiene ruta de lanzamiento. El guard Hook se conserva.

## Quality gates ejecutados

Desde `SDLC/05_Construccion/aplicacion/`:

| Comando | Resultado |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 196 passed, 7 ignored (host/network/opt-in) |

Desde `apps/lxmi-desktop/`:

| Comando | Resultado |
|---|---|
| `npm run typecheck` | PASS |
| `npm run lint` | PASS |
| `npm run format:check` | PASS tras aplicar Prettier al panel editado |
| `npm run build` | PASS — typecheck y Vite production build |

| `git diff --check` | PASS |

## No ejecutado

- **NO COMPROBADO:** inspección visual/manual desde una ventana Tauri nativa.
- No se ejecutó ningún proceso Steam/Proton/Wine/ZZZ/loader; no hubo test host de runtime.
- No se modificaron Steam, el juego, `compatdata/4162040`, `pfx` ni el contenido de runtime administrado.
