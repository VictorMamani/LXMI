# LXMI — aplicación

**LXMI 0.5.2 — Tauri 2 + React/TypeScript + Rust.** Workspace crates: `lxmi-core`, `lxmi-steam`, `lxmi-proton`, `lxmi-runtime` y `lxmi-xxmi`. Discovery, planificación y dry-run sobre Steam/juego siguen read-only. La consulta/descarga oficial se inicia explícitamente; extracción e import solo escriben bajo cache XDG y `$XDG_DATA_HOME/lxmi`. No hay SQLite, executor ni ejecución de procesos/contenido importado. No se instala al juego.

En LXMI-0.1 a 0.4, la app usó **Tauri 2 + React + TypeScript + Rust** para discovery y planificación de solo lectura. El workspace añadió en 0.5 `lxmi-xxmi`. No usa SQLite ni escribe en Steam, juegos, compatibility tools o prefixes; el import explícito sí conserva paquetes bajo el storage privado de LXMI.

## Requisitos de desarrollo

- Ubuntu con dependencias nativas de Tauri 2 instaladas. Consulta la [guía oficial de prerrequisitos Linux de Tauri](https://v2.tauri.app/start/prerequisites/); incluye WebKitGTK 4.1, GTK, OpenSSL y herramientas de compilación.
- Rust stable mediante rustup, con rustfmt y clippy.
- Node.js 22.13 o posterior y npm.
- Una sesión de escritorio Linux para abrir la ventana de Tauri.

No se necesitan credenciales, permisos root, Steam en ejecución, juegos instalados ni prefixes para compilar y probar el detector.

## Estructura

| Ruta | Responsabilidad |
|---|---|
| apps/lxmi-desktop/ | Interfaz React y paquete Tauri |
| apps/lxmi-desktop/src-tauri/ | Commands adaptadores y configuración de escritorio |
| crates/lxmi-core/ | Información del sistema, registro de juegos y modelos de instalación/compatdata |
| crates/lxmi-steam/ | Steam roots/bibliotecas, parser KeyValues, manifests, game scanner y compatdata discovery |
| crates/lxmi-proton/ | Discovery de compatibility tools Steam/custom; parser Valve KeyValues reutilizado; metadata, clasificación y versión |
| crates/lxmi-runtime/ | Plan declarativo por juego; agrega observaciones de instalación, Proton candidates, compatdata/prefix, requirements, evidence, issues y readiness |
| crates/lxmi-xxmi/ | Modelos de paquete, detección acotada, SHA-256, importación a storage administrado y plan sin executor |

## Instalar dependencias y ejecutar

Desde este directorio:

```bash
cd apps/lxmi-desktop
npm ci
npm run tauri:dev
```

**Scan Steam** inspecciona roots, bibliotecas, manifests, `compatdata` y metadata de compatibility tools en modo de solo lectura. El catálogo reconoce Wuthering Waves por AppID `3513350` y Zenless Zone Zero por `4162040`; en ZZZ también busca `ZenlessZoneZero.exe` y `ZenlessZoneZeroBeta.exe` dentro de un árbol acotado sin seguir symlinks. Proton se valida mediante metadata estructural y un entrypoint `proton` regular; Steam Linux Runtime se clasifica aparte. Si se encuentra `pfx`, LXMI lo muestra como candidato. La app no determina qué Proton seleccionó Steam ni afirma que el juego sea compatible.

Las ubicaciones y metadata se basan en [Valve Proton](https://github.com/ValveSoftware/Proton), la [plantilla oficial `compatibilitytool.vdf`](https://github.com/ValveSoftware/Proton/blob/proton_11.0/compatibilitytool.vdf.template) y la [documentación oficial Steam Runtime](https://github.com/ValveSoftware/steam-runtime/blob/master/doc/reporting-steamlinuxruntime-bugs.md). LXMI no ejecuta esos runtimes.

## Validaciones

Desde la raíz de esta aplicación, con los prerrequisitos nativos de Tauri instalados:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Desde apps/lxmi-desktop/:

```bash
npm run typecheck
npm run lint
npm run format:check
npm run build
```

La suite Rust usa fixtures y directorios temporales. Una comprobación local separada ejecutó los scanners contra Steam en solo lectura. No modifica la instalación personal de Steam.

## LXMI-0.1 a 0.4 — discovery y planificación

Implementa información del sistema, detección Steam, parser KeyValues reutilizado por VDF/ACF y metadata de tools, catálogo Wuthering Waves, inspección pasiva de compatdata/pfx, discovery de Proton y Steam Linux Runtime, y planificación efímera por juego con requisitos y evidencia. `runtime discovery != runtime selection`; LXMI conserva la selección como desconocida sin evidencia fiable, incluso con un único Proton candidate. `runtime selection != launching`.

La evaluación no demuestra que Steam elija un candidato, que el prefix esté sano, que el juego sea compatible ni que XXMI esté listo. La UI muestra un resumen de readiness y una sección expandible de evidencia. El escaneo sigue siendo de solo lectura; no ejecuta Steam, Proton, Wine ni el juego y no modifica Steam, launch options, compatdata o pfx.

## Verificación de LXMI-0.4

Pasaron `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` y `cargo test --workspace` (69 tests). En `apps/lxmi-desktop/` pasaron `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`.

`npm run tauri:dev` inició la aplicación y ejecutó el scan contra Steam local en modo de solo lectura. Detectó Wuthering Waves con directorio presente, compatdata/pfx ausentes, dos candidatos Proton (Experimental y Hotfix), tres Steam Linux Runtime, selección desconocida y readiness `NeedsInitialization`. Un symlink irrelevante `Steam.dll` fue ignorado sin seguirlo y no hizo que el scan quedara parcial. No se encontró una herramienta custom.

Los tests del planner usan datos sintéticos. El scan real es una comprobación de integración de este equipo; no demuestra selección real de Proton, estado saludable de un prefix, compatibilidad del juego ni preparación de XXMI. No se ejecutó Steam, Proton, Wine ni el juego, y no hubo escrituras en Steam/juego/compatdata/pfx.

## LXMI-0.5: paquetes XXMI/WWMI

Se agrega `crates/lxmi-xxmi`: reconoce WWMIv1 por configuración/recursos y XXMI Libraries como paquete separado por sus tres DLL más `Manifest.json`. Soporta carpetas locales, no ZIP ni descarga. SHA-256 comprueba cambios respecto al import; no autentica firma/origen upstream. Usa storage XDG, staging acotado y manifest LXMI externo al payload.

La UI expone inspección, import a storage y revisión de plan. La inspección del juego usa ubicaciones candidatas limitadas. El plan propone destinos dentro del storage administrado, muestra requisitos faltantes/desconocidos, `executable=false` y no tiene apply. No se determina compatibilidad de ejecución. La ventana Tauri nativa inició; la interacción IPC del import no se recorrió clic a clic.

Upstream/layout/licencias: `../../01_Descubrimiento/ecosistema_xxmi_wwmi.md`. Decisiones: `../../04_Arquitectura_y_seguridad/ADR/`. El asset upstream `Manifest.json` de XXMI Libraries se publica aparte y debe colocarse junto a las DLL antes de importar. El ejemplo de desarrollo `cargo run -p lxmi-xxmi --example local_review -- <store-existente> [WWMI-dir] [XXMI-libs-dir]` escribe solo en el root explícito suministrado; no es instalador.

Validación detallada: `../../06_Verificacion/verificacion_0_5.md`.

En el scan local del 2026-09-28 no apareció el manifest de Wuthering Waves; los directorios Proton encontrados no contenían metadata/entrypoint suficiente para validarlos y no se halló compatdata. El planner quedó `Blocked`; este estado describe el host en esa fecha y puede cambiar.

## LXMI-0.5.1: Zenless Zone Zero / ZZMI

`lxmi-core` y `lxmi-steam` registran ZZZ por AppID `4162040`, buscan sus ejecutables esperados sin seguir symlinks y relacionan el manifest Steam con compatdata. `lxmi-xxmi` reconoce ZZMI como integración de ZZZ, valida anclas estructurales de `d3dx.ini` y `Core/ZZMI/main.ini`, conserva la versión raw, impide intercambiar ZZMI/WWMI entre juegos y modela XXMI Libraries como paquete separado requerido. Importar una carpeta usa el mismo staging, hash y storage que LXMI 0.5.

La investigación upstream y los límites de licencia/compatibilidad están en `../../01_Descubrimiento/ecosistema_xxmi_zzmi.md`. El SHA-256 local no autentica al publisher; LXMI no verifica firmas upstream. No se descargó ni importó una release real en este incremento.

**Validación local read-only:** manifest Steam local confirma AppID `4162040`; directorio y `ZenlessZoneZero.exe` existen; `compatdata/4162040/pfx` existe como candidato. El command snapshot reportó un candidato Proton, selección `unknown` y readiness `incomplete`. No se lanzó el juego ni se escribió en Steam, ZZZ o el prefix. Tauri dev inició y compiló, pero el click manual de Scan en la ventana no fue comprobado. Ver `../../06_Verificacion/verificacion_0_5_1.md`.

## LXMI-0.5.2: releases oficiales y dry-run

`lxmi-xxmi` ofrece `ReleaseProvider` offline-testable y adaptador GitHub con allowlist exacta para ZZMI y XXMI Libraries. Guarda identity reproducible (release ID/tag/commit, asset, hashes y timestamps), descarga con streaming/límites a XDG cache, verifica firma ECDSA P-384/SHA-256 usando claves fijadas de XXMI Launcher, extrae ZIP con límites a staging y promueve paquetes validados al storage privado. En Libraries también verifica las firmas DLL declaradas en el `Manifest.json` separado. El SHA-256 local continúa siendo integridad, no autenticidad.

La UI permite consultar releases solo tras click explícito, elegir el tag consultado, descargar/verificar/importar ZZMI y Libraries por separado y revisar el plan/dry-run. Mapeo relativo sigue el `importer_path` configurable de XXMI; su raíz efectiva no se conoce automáticamente. La carpeta del ejecutable ZZZ es un candidato de comparación, no un target aprobado. `apply_allowed=false`; no se escribe en juego/Steam/compatdata/prefix y no se ejecuta contenido. Steam/Linux/Proton sigue sin verificar. Las releases recomiendan XXMI Launcher para instalación; LXMI no reemplaza ese flujo.

Pins, digests, commits, firmas, inventario y fuentes upstream: `../../01_Descubrimiento/adquisicion_paquetes_xxmi_0_5_2.md`. Security/decision: `../../04_Arquitectura_y_seguridad/ADR/024_official_upstream_package_trust.md`. El estado real de validación y UI está en `../../06_Verificacion/verificacion_0_5_2.md`.

Durante la validación local, la consulta de LXMI seleccionó ZZMI `v1.5.0` (release `393483881`) y XXMI Libraries `v1.1.7` (release `387957029`). Ambos se descargaron, autenticaron e importaron; la UI Tauri recorrió también el plan de 49 destinos sin aplicarlo. La autenticidad upstream no verifica compatibilidad con Steam/Linux/Proton. La prueba HTTP está ignorada en la suite normal y usa store/cache temporales con `LXMI_CONFIRM_OFFICIAL_DOWNLOAD_AND_STORE=YES`.
