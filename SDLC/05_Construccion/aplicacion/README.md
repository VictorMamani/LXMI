# LXMI — aplicación

**LXMI 0.6.0 — Tauri 2 + React/TypeScript + Rust.** El workspace contiene `lxmi-core`, `lxmi-steam`, `lxmi-proton`, `lxmi-runtime`, `lxmi-xxmi`, `lxmi-bridge`, `lxmi-bridge-protocol` y `tools/lxmi-bridge-helper`. Discovery, planificación y topología siguen read-only. La consulta/descarga requiere acción explícita; import y ensamblado solo escriben bajo cache/storage privado XDG de LXMI. El bridge de diagnóstico no se ejecuta automáticamente; usa un helper inocuo y un prefix de prueba LXMI aislado. No hay SQLite, launcher de juego, injector, ni instalación en ZZZ.

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
| crates/lxmi-xxmi/ | Paquetes y provenance, import seguro, assembler de ZZMI + XXMI Libraries a XDG, configuración derivada, discovery de `dosdevices` y plan de topología sin executor |
| crates/lxmi-bridge/ | Validación de helper/runtime, test-prefix aislado, proceso Proton explícito y bounded, handshake y path/hash result |
| crates/lxmi-bridge-protocol/ | Request/response JSON versionado compartido por LXMI y helper |
| tools/lxmi-bridge-helper/ | Helper Windows inocuo: inspecciona únicamente un marker JSON dentro del storage administrado de LXMI |

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

## LXMI-0.5.2: releases oficiales y dry-run histórico

`lxmi-xxmi` ofrece `ReleaseProvider` offline-testable y adaptador GitHub con allowlist exacta para ZZMI y XXMI Libraries. Guarda identity reproducible (release ID/tag/commit, asset, hashes y timestamps), descarga con streaming/límites a XDG cache, verifica firma ECDSA P-384/SHA-256 usando claves fijadas de XXMI Launcher, extrae ZIP con límites a staging y promueve paquetes validados al storage privado. En Libraries también verifica las firmas DLL declaradas en el `Manifest.json` separado. El SHA-256 local continúa siendo integridad, no autenticidad.

La UI permite consultar releases solo tras click explícito, elegir el tag consultado, descargar/verificar/importar ZZMI y Libraries por separado y revisar el plan/dry-run. Ese dry-run de 49 rutas contra la carpeta candidata derivada del ejecutable es **comparación histórica solamente**: no representa un target autorizado ni la topología activa de LXMI 0.5.3. No se escribe en juego/Steam/compatdata/prefix y no se ejecuta contenido.

Pins, digests, commits, firmas, inventario y fuentes upstream: `../../01_Descubrimiento/adquisicion_paquetes_xxmi_0_5_2.md`. Security/decision: `../../04_Arquitectura_y_seguridad/ADR/024_official_upstream_package_trust.md`. El estado real de validación y UI está en `../../06_Verificacion/verificacion_0_5_2.md`.

Durante la validación local, la consulta de LXMI seleccionó ZZMI `v1.5.0` (release `393483881`) y XXMI Libraries `v1.1.7` (release `387957029`). Ambos se descargaron, autenticaron e importaron; la UI Tauri recorrió también el plan de 49 destinos sin aplicarlo. La autenticidad upstream no verifica compatibilidad con Steam/Linux/Proton. La prueba HTTP está ignorada en la suite normal y usa store/cache temporales con `LXMI_CONFIRM_OFFICIAL_DOWNLOAD_AND_STORE=YES`.

## LXMI-0.5.3: managed importer runtime y launch topology

`lxmi-xxmi` interpreta `App.Root` e `importer_path` con el modelo upstream: `ZZMI/` se resuelve relativo al root del launcher. LXMI compone paquetes ZZMI y XXMI Libraries verificados en `$XDG_DATA_HOME/lxmi/runtimes/zenless-zone-zero/zzmi/<runtime-id>/`; los paquetes originales permanecen inmutables. `d3dx.ini` se deriva durante staging para el target ZZZ observado y conserva `loader = XXMI Launcher.exe`. Las DLLs que upstream despliega bajo el importer se copian dentro de la raíz LXMI; `3dmloader.dll` permanece en el paquete Libraries y se referencia por ruta/hash.

La UI permite revisar el plan de ensamblado antes de preparar la copia privada y después inspeccionar una `LaunchTopologyPlan`. La inspección de `pfx/dosdevices` solo lee los symlinks inmediatos; no los crea ni recorre sus destinos. La estrategia de helper, el requisito de compartir prefix y la selección Proton siguen `Unknown`. Ningún proceso se lanza ni se modifica ZZZ/Steam/compatdata/prefix. El mapping 0.5.2 queda etiquetado como histórico, sin inspección activa del directorio del ejecutable como target.

Fuentes y arquitectura: `../../01_Descubrimiento/topologia_runtime_xxmi_linux.md`; decisión bridge: `../../04_Arquitectura_y_seguridad/ADR/025_native_lxmi_windows_runtime_bridge.md`. Verificación: `../../06_Verificacion/verificacion_0_5_3.md`.

## LXMI-0.6: Controlled Windows Runtime Bridge

La sección Advanced del panel ZZMI puede inspeccionar el helper LXMI staged, el runtime ZZMI administrado y las herramientas Proton elegibles. Elegir un Proton ahí significa **Bridge test runtime** únicamente: no escribe ni infiere qué Proton seleccionará Steam para ZZZ. La acción está deshabilitada hasta que existan paquetes seleccionados, runtime ensamblado, helper con SHA-256 coincidente y un candidato Proton que el backend vuelve a validar.

El helper Windows usa protocolo stdin/stdout JSON v1, nonce por request, respuesta estructurada, límites de 64 KiB por request, 1 MiB por stream y 20 segundos. Solo puede leer runtime-manifest.json debajo del root administrado. No enumera procesos, no abre ZZZ, no carga DLL ni modifica el juego. El nonce correlaciona; no autentica. SHA-256 es integridad local, no firma del publisher.

### Compilar y preparar el helper (optativo)

Requiere el target Rust x86_64-pc-windows-gnu y MinGW-w64 (`gcc-mingw-w64-x86-64`, `mingw-w64-x86-64-dev`) en el host. Desde la raíz de esta aplicación:

~~~bash
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build --manifest-path tools/lxmi-bridge-helper/Cargo.toml \
  --target x86_64-pc-windows-gnu --release
bash tools/install-bridge-helper.sh
~~~

El segundo comando copia el binario a helpers/ y genera su sidecar SHA-256 bajo el storage LXMI definido por XDG. La UI no descarga ni compila helpers por su cuenta. No usar sudo para preparar el toolchain.

### Efectos del test

Al pulsar Run Runtime Bridge Test, LXMI invoca el entrypoint Proton con `runinprefix` mediante argv, un entorno limpio/allowlisted y el CompatData:

~~~text
$XDG_DATA_HOME/lxmi/test-prefixes/bridge-v1/compatdata
~~~

Proton puede crear/actualizar pfx allí y mantener su propia distribución, por ejemplo procesar fixups o retirar un dist/ legacy. La UI obliga a aceptar estos efectos. Nunca se pasa compatdata/4162040. **Host test comprobado:** Proton Experimental ejecutó el helper, verificó el nonce, leyó el runtime marker por su ruta Windows y obtuvo el mismo SHA-256. Un path inexistente produjo una respuesta negativa estructurada. El `.exe` staged es PE32+ x86-64 con SHA-256 `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2`. La ventana Tauri nativa mostró el resultado; la selección Proton del juego sigue `unknown` y no se afirma compatibilidad ZZMI/ZZZ/Linux.

El test escribió en el prefix/contexto privado `$XDG_DATA_HOME/lxmi/test-prefixes/bridge-v1/compatdata` y modificó el timestamp de `dist.lock` de Proton Experimental. No alteró `compatdata/4162040`, el ejecutable ZZZ ni los targets de juego vigilados. Ver el detalle completo en `../../06_Verificacion/verificacion_0_6.md`.

Fuentes Valve fijadas y detalles de invocación: ../../01_Descubrimiento/bridge_linux_windows.md. Decisión: ../../04_Arquitectura_y_seguridad/ADR/026_lxmi_windows_bridge_protocol.md. Verificación y limitaciones: ../../06_Verificacion/verificacion_0_6.md.
