# Verificación LXMI 0.5.3 — Managed Importer Runtime & Launch Topology

**Fecha:** 2026-09-28
**Repositorio:** `main`, root `04_Proyectos/19_LXMI`, remote `origin` `git@github.com:VictorMamani/LXMI.git`.
**Base Git:** `9646631 feat: complete official ZZMI package verification and dry-run`.
**Alcance:** ensamble en XDG privado y discovery read-only de topología. Sin lanzamiento, inyección, apply o escrituras en Steam/ZZZ/compatdata/prefix.

## Resultado host-only

**COMPROBADO:** el test opt-in encontró Zenless Zone Zero Steam, AppID `4162040`, el executable `ZenlessZoneZero.exe` configurado y un prefix candidato en `compatdata/4162040/pfx`. Encontró Proton Experimental como candidato disponible, pero conservó `selected_proton = unknown`.

**COMPROBADO:** se verificaron desde storage gestionado los paquetes oficiales ZZMI v1.5.0 y XXMI Libraries v1.1.7. El ensamblado resultante tiene 49 archivos y se ubicó en:

```text
/home/university/.local/share/lxmi/runtimes/zenless-zone-zero/zzmi/zzmi-6dc676fdfc0d30f7e69de43b9123936e7750b0e58baadb384018815a2891f052/
```

El importer se resolvió como `App.Root/ZZMI/`; incluye `Mods/`, configuración derivada y `runtime-manifest.json` separado del payload. La ruta Linux del importer se mapeó lexicalmente por `Z:` como:

```text
Z:\home\university\.local\share\lxmi\runtimes\zenless-zone-zero\zzmi\zzmi-6dc676fdfc0d30f7e69de43b9123936e7750b0e58baadb384018815a2891f052\ZZMI
```

El prefix reportó `c:` → `../drive_c`, `s:` → Steam y `z:` → `/`; los enlaces se leyeron de forma no recursiva. `SamePrefixRequirement = Unknown`, `LoaderStrategy = Unknown`, `execution_enabled = false`, `external_files_modified = false`, readiness `PlannedIncomplete`.

La prueba comparó antes/después los estados de `d3d11.dll`, `d3dcompiler_47.dll`, `d3dx.ini` y `ZZMI` bajo la carpeta que contiene el executable, y el `DosDevicesReport` de `pfx/dosdevices`: no cambiaron. También volvió a validar los inventarios SHA-256 de ambos paquetes fuente.

## Upstream

**UPSTREAM VERIFIED:** la evidencia registrada en [topología runtime XXMI Linux](../01_Descubrimiento/topologia_runtime_xxmi_linux.md) fija `SpectrumQT/XXMI-Launcher` v2.2.1, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`; y ZZMI v1.5.0, commit `e59f87047cd405c5db5476d3b1b499574bc43d67`. `importer_path` relativo se combina con `App.Root`, ZZMI declara `ZZMI/`, y `loader = XXMI Launcher.exe` se conserva como identidad Windows allowlisted. El dry-run 0.5.2 contra el directorio candidato de ZZZ es histórico y no un target aprobado.

**UNKNOWN:** no se eligió entre launcher portable, helper upstream o helper propio. No se determinó que helper/juego deban compartir prefix ni se verificó ZZZ Steam + Linux/Proton + ZZMI.

## Tests y quality gates

**FIXTURE / OFFLINE:** `cargo test --workspace` pasa con fixtures sintéticos; las pruebas unitarias no dependen de Steam ni de Internet. La prueba de descarga upstream, la comparación upstream real y el scan/ensamble host-only permanecen opt-in/ignorados en el run normal.

**COMPROBADO:** la prueba host-only opt-in se ejecutó con:

```bash
LXMI_CONFIRM_REAL_ZZZ_ASSEMBLY=YES \
LXMI_REAL_ZZZ_EXECUTABLE='/home/university/.local/share/Steam/steamapps/common/Zenless Zone Zero/games/ZenlessZoneZero Game/ZenlessZoneZero.exe' \
cargo test -p lxmi-xxmi --test upstream_real \
  assembles_verified_zzmi_runtime_and_inspects_local_zzz_topology_without_game_writes \
  -- --ignored --exact --nocapture
```

El resultado fue `1 passed`; reportó 49 archivos, mapping del importer por `Z:`, selección Proton desconocida, sin ejecución y sin escrituras externas.

**COMPROBADO:** `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` y `cargo test --workspace` terminaron con exit code 0. El workspace tuvo 149 tests aprobados y 5 ignorados (host/network/opt-in). La prueba real de ensamble se ejecutó por separado y aprobó.

**COMPROBADO:** `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build` terminaron con exit code 0; Vite construyó el bundle de producción.

Había un `tauri dev` y una ventana LXMI activos antes de la validación; no se lanzó un segundo servidor ni se cerró la sesión existente. `cargo check --workspace` incluyó el crate Tauri. No se ejecutó nuevamente `npm run tauri:dev` desde este turno.

## UI manual

**NO COMPROBADO:** no se completó un flujo de clicks en la ventana nativa Tauri. La página Vite se pudo renderizar en un browser externo, pero no tiene el bridge Tauri: la carga básica de sistema falló y Scan Steam quedó deshabilitado. Esa vista no se considera una validación funcional de la UI nativa. Se observó el copy `v0.5.3` y los botones del panel nuevo, pero falta validar revisar/ensamblar/topología mediante comandos desde la ventana.

## Seguridad y límites

- **COMPROBADO:** el test guardó el runtime solo bajo storage XDG de LXMI y comprobó que el storage no se solapa con Steam, juego o prefix.
- **COMPROBADO:** paquetes fuente revalidados; su inventario no cambió.
- **COMPROBADO:** las rutas candidatas revisadas de ZZZ y los links de `dosdevices` quedaron iguales antes/después.
- **COMPROBADO:** no se ejecutó Steam, Proton, Wine, launcher, helper, juego ni payload.
- **COMPROBADO:** `apply`/executor no existe en este incremento.
- **NO COMPROBADO:** compatibilidad o inyección en ZZZ bajo Proton; selección de Proton; helper y requisito de mismo prefix; operación manual de los nuevos botones nativos.

La prueba host-only requiere opt-in porque materializa el runtime privado una vez; al repetirla, el ensamblado verifica/reutiliza el runtime existente. No borra runtime ni contenido.
