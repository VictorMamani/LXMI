# Verificación LXMI 0.5.1 — ZZZ / ZZMI

**Fecha:** 2026-09-28.
**Alcance:** detección, modelo, validación estructural y planificación. No hay apply.

## Resultado comprobado en el host

El command de scan de la aplicación, ejecutado mediante una prueba local marcada `#[ignore]` para que no dependa del host en la suite estándar, observó:

| Dato | Resultado |
|---|---|
| Distribución | Steam |
| Juego / AppID | Zenless Zone Zero / `4162040` |
| Manifest local | Encontrado y parseado |
| Directorio derivado de `installdir` | Presente |
| Ejecutable | `ZenlessZoneZero.exe` encontrado en `games/ZenlessZoneZero Game/` |
| `compatdata/4162040` | Presente |
| `compatdata/4162040/pfx` | Presente, solo candidato |
| Candidatos Proton | 1 en el scan; no implica selección |
| Runtime seleccionado para el juego | `unknown` |
| Readiness del plan de plataforma | `incomplete` |

La prueba no ejecuta el juego, Steam, Proton, Wine ni archivos de la instalación. No escribe en Steam, juego, compatdata ni pfx. No se hizo snapshot/hash previo y posterior del árbol real; la garantía proviene de la ruta de código read-only que se ejecutó y de tests de invariantes con fixtures.

El Tauri dev server arrancó y compiló el binario desktop v0.5.1. El click manual en la ventana y el flujo IPC interactivo no se comprobaron en esta sesión. La prueba host-only ejecuta la misma función de snapshot usada por `scan_steam`, no una automatización de la ventana.

## Paquete e integración

- Los tests de integración usan payloads **FIXTURE** sintéticos sin DLL, scripts ni datos de usuario reales.
- La lista mínima ZZMI está contrastada contra `d3dx.ini` y los includes del árbol upstream revisado, pero no se importó una release real en 0.5.1.
- No hay package ZZMI real verificado en el storage administrado de este host; no se afirma `ZZMI available` por fixture.
- XXMI Libraries es una dependencia independiente. Fixtures prueban la relación y el estado faltante.
- Steam support y compatibilidad Linux/Proton son `Unverified`; el plan no es `ReadyToInstall` con evidencia incompleta.
- No se modificó el prototipo visual separado.

## Ejecutar comprobaciones de host

Desde `SDLC/05_Construccion/aplicacion/`:

```bash
cargo test -p lxmi-desktop local_steam_scan_finds_zzz_without_inferring_proton_selection -- --ignored --nocapture
```

Esta prueba está ignorada por defecto porque exige que ZZZ siga instalada en Steam en el host. Solo lee discovery y verifica que la selección de runtime permanezca desconocida.

## Checklist manual no destructiva

| Paso | Resultado actual |
|---|---|
| Abrir LXMI desktop | Tauri dev compiló e inició; revisión visual del estado ZZZ: NO COMPROBADA |
| Pulsar `Scan Steam` | NO COMPROBADO con interacción real; command host-only sí ejecutó scan real |
| Confirmar ZZZ y AppID local | COMPROBADO por manifest local y prueba host-only |
| Confirmar ruta y ejecutable | COMPROBADO por filesystem local y prueba host-only |
| Confirmar compatdata/prefix | COMPROBADO como rutas presentes; no se abre/valida la salud de archivos internos |
| Abrir panel XXMI/ZZMI | NO COMPROBADO con interacción real |
| Importar paquete ZZMI local | NO COMPROBADO; requiere payload local. No se descargó nada |
| Revisar el plan | Planes sintéticos cubiertos; plan con paquetes upstream reales: NO COMPROBADO |
| Confirmar que no cambió el juego | No se ejecutó ninguna mutación contra el juego; no se hizo comparación hash/snapshot real |

Antes de una futura instalación real, repetir y guardar evidencias de los destinos/archivos antes y después; todavía no hay operación apply en LXMI.

## Gates finales (2026-09-28)

| Comando | Resultado |
|---|---|
| `git diff --check` | Pasó |
| `cargo fmt --all -- --check` | Pasó |
| `cargo check --workspace` | Pasó |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Pasó |
| `cargo test --workspace` | 126 pasaron; 1 prueba host-only ignorada por defecto |
| `cargo test -p lxmi-desktop local_steam_scan_finds_zzz_without_inferring_proton_selection -- --ignored --nocapture` | Pasó en el host; ZZZ Steam detectada, selección Proton `unknown` |
| `npm run typecheck` | Pasó |
| `npm run lint` | Pasó |
| `npm run format:check` | Pasó |
| `npm run build` | Pasó |
| `npm run tauri:dev` | Compiló e inició la aplicación; interacción manual de la ventana no verificada |

No quedó un servidor de desarrollo Tauri/Vite en ejecución al cerrar la validación.
