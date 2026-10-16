# LXMI — aplicación

**LXMI 0.8.7 — Tauri 2 + React/TypeScript + Rust.** El workspace contiene `lxmi-core`, `lxmi-steam`, `lxmi-proton`, `lxmi-runtime`, `lxmi-xxmi`, `lxmi-bridge`, `lxmi-bridge-protocol` y `tools/`. Una observación nativa de una ejecución manual de ZZZ identificó Proton Experimental para esa sesión, AppID `4162040`, compatdata coincidente y prefix observado. El mapping configurado por AppID no apareció en la fuente inspeccionada; el mapping global sigue separado. `GameRuntimePlan.selected` continúa `Unknown`, estrategia `NoDecision`, readiness `NeedsEvidence` y gate cerrado. Context Lab ejecuta solo peers propios con consentimiento, en prefixes privados de LXMI. El gate de carga con mods sigue cerrado. El nuevo botón «Jugar sin mods» solicita el lanzamiento normal mediante Steam; esta acción permite a Steam realizar su inicio habitual y no se ejecutó durante la verificación del gestor. El observer sigue siendo read-only.

En LXMI-0.1 a 0.4, la app usó **Tauri 2 + React + TypeScript + Rust** para discovery y planificación de solo lectura. El workspace añadió en 0.5 `lxmi-xxmi`. No usa SQLite ni escribe en Steam, juegos, compatibility tools o prefixes; el import explícito sí conserva paquetes bajo el storage privado de LXMI.

## Jugar con mods (ZZZ)

`crates/lxmi-launch` inicia Zenless Zone Zero con el perfil aplicado mediante el Hook oficial de XXMI: copia de arranque verificada en `$XDG_DATA_HOME/lxmi/launches/`, contexto Steam de HoYoPlay leído de procfs (AppID, compatdata, prefix, Proton y Steam Linux Runtime), helper `tools/lxmi-mod-loader/loader.c` dentro de ese contexto, lanzado mediante el servicio de Steam (`steam-runtime-launch-client --alongside-steam`) porque AppArmor de Ubuntu impide a una app sin perfil crear el contenedor del SLR, y retirada del hook en cuanto ZZMI está en el juego. `build.rs` compila el helper con `x86_64-w64-mingw32-gcc` y lo incrusta; sin MinGW la app compila y el botón queda deshabilitado. Comandos Tauri: `start_game_with_mods`, `game_with_mods_status`, `cancel_game_with_mods`. Prueba de humo sin procesos: `cargo run -p lxmi-desktop --example prepare_mod_launch`. [Manual de uso](../../../MANUAL_DE_USO.md) · [ADR-032](../../04_Arquitectura_y_seguridad/ADR/032_jugar_con_mods_hook_oficial.md).

## Gestor local de mods y perfiles

La interfaz sigue el prototipo existente: **Inicio**, **Mods**, **Perfiles** y **Compatibilidad**, con los diagnósticos anteriores accesibles. Permite importar carpetas/ZIP, guardar selecciones por juego y preparar copias privadas. **Compatibilidad** ensambla WWMI o ZZMI con XXMI Libraries autenticadas y permite revisar/aplicar/revertir un perfil mediante runtimes derivados inmutables. WuWa y ZZZ usan bibliotecas separadas. La selección todavía no acredita mods cargados dentro del juego ni permite cambios live. [Guía de uso, almacenamiento y límites](../gestor_mods_local.md) · [Verificación](../../06_Verificacion/verificacion_gestor_mods_local.md).

## Requisitos de desarrollo

- Ubuntu con dependencias nativas de Tauri 2 instaladas. Consulta la [guía oficial de prerrequisitos Linux de Tauri](https://v2.tauri.app/start/prerequisites/); incluye WebKitGTK 4.1, GTK, OpenSSL y herramientas de compilación.
- Rust stable mediante rustup, con rustfmt y clippy.
- Node.js 22.13 o posterior y npm.
- Una sesión de escritorio Linux para abrir la ventana de Tauri.
- MinGW-w64 (`x86_64-w64-mingw32-gcc`) para incluir el cargador de «Jugar con mods»; sin él la app compila con esa función deshabilitada.

No se necesitan credenciales, permisos root, Steam en ejecución, juegos instalados ni prefixes para compilar y probar el detector.

## Estructura

| Ruta | Responsabilidad |
|---|---|
| apps/lxmi-desktop/ | Interfaz React y paquete Tauri |
| apps/lxmi-desktop/src-tauri/ | Commands adaptadores y configuración de escritorio |
| crates/lxmi-core/ | Información del sistema, registro de juegos y modelos de instalación/compatdata |
| crates/lxmi-steam/ | Steam roots/bibliotecas, parser KeyValues, manifests, game scanner, compatdata y lectura limitada de `CompatToolMapping` |
| crates/lxmi-proton/ | Discovery de compatibility tools Steam/custom; parser Valve KeyValues reutilizado; metadata, clasificación y versión |
| crates/lxmi-runtime/ | Plan declarativo por juego; agrega observaciones de instalación, Proton candidates, compatdata/prefix, requirements, evidence, issues y readiness |
| crates/lxmi-xxmi/ | Paquetes y provenance, import seguro, assembler de ZZMI + XXMI Libraries a XDG, configuración derivada, discovery de `dosdevices` y plan de topología sin executor |
| crates/lxmi-launch/ | «Jugar con mods»: contexto Steam por procfs, copia de arranque, `[Loader]` y máquina de estados del Hook oficial |
| crates/lxmi-bridge/ | Validación de helper/runtime, test-prefix aislado, proceso Proton explícito y bounded, handshake y path/hash result |
| crates/lxmi-bridge-protocol/ | Request/response JSON versionado compartido por LXMI y helper |
| tools/lxmi-bridge-helper/ | Helper Windows inocuo: inspecciona únicamente un marker JSON dentro del storage administrado de LXMI |
| tools/mod-repair/ | `repair_mod.py`: copia reparada de un mod (secciones duplicadas, recursos ausentes, `.ini` rotos, texturas 8K→4K); requiere Pillow y etcpak |
| tools/lxmi-mod-loader/ | Helper Windows de producción (`HookLibrary`/`UnhookLibrary`), compilado e incrustado por `build.rs` |
| tools/lxmi-loader-lab/ | Código MinGW del runner, proceso test y DLL inofensiva; el `3dmloader.dll` upstream no se incluye ni redistribuye |

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

## LXMI-0.7: Upstream Loader Compatibility Experiment

La sección **Advanced · Loader Lab** materializa un test target y una DLL inocua compilados para Windows x86-64 con MinGW. El backend verifica el package XXMI Libraries autenticado, su `3dmloader.dll`, los hashes staged y un `LoaderExperimentPlan` de rutas fijas bajo `$XDG_DATA_HOME/lxmi/tests/loader-v1/`. El executor solo admite Proton elegido expresamente para test, usa `runinprefix`, argv directo, 30 s de timeout y hasta 1 MiB por stream. La prueba puede escribir únicamente en `$XDG_DATA_HOME/lxmi/test-prefixes/loader-v1/compatdata` tras aceptación explícita.

### Construir los ejecutables de laboratorio

Requiere MinGW-w64 x86-64 (`gcc-mingw-w64-x86-64`, `mingw-w64-x86-64-dev`) ya instalado en Ubuntu. Desde este directorio:

```bash
bash tools/lxmi-loader-lab/build.sh
```

El script genera `lxmi-loader-test-runner.exe`, `lxmi-loader-test-target.exe` y `lxmi-loader-test.dll` dentro de `target/x86_64-pc-windows-gnu/release/lxmi-loader-lab/`. En la app se pulsa **Inspeccionar Loader Lab** para verificar provenance y staging. LXMI copia los binarios de test y el `3dmloader.dll` de la release verificada a storage XDG propio; no incluye el binario upstream en Git.

**HOST TEST comprobado:** Proton Experimental explícitamente seleccionado ejecutó baseline sin loader, Direct Inject positivo con marker/nonce y negativos de target ausente, DLL ausente y nonce incorrecto. La ventana Tauri nativa recorrió baseline, positivo y target ausente. Este experimento solo valida el export upstream en el proceso de prueba propio de LXMI; no es el Hook predeterminado de ZZMI y no verifica ZZMI, ZZZ, Steam/Linux/Proton para el juego ni mods. No interactúa con anti-cheat, Steam, juegos ni procesos ajenos al target de prueba.

Fuentes fijadas, licencia, diferencias de modo y capabilities: `../../01_Descubrimiento/loader_xxmi.md`; decisión: `../../04_Arquitectura_y_seguridad/ADR/027_xxmi_loader_strategy_for_lxmi.md`; resultados: `../../06_Verificacion/verificacion_0_7.md`.

## LXMI-0.8.1 — estrategia loader y readiness de ZZZ

La investigación fijada a XXMI Launcher v2.2.1 confirmó que ZZMI hereda `use_hook=true` para el flujo estándar; Custom Launch expone Hook, Inject y Bypass. Inject utiliza la ruta genérica Direct Inject de `3dmloader.dll`, pero esto no demuestra compatibilidad con ZZMI/ZZZ en Linux/Proton. La evidencia upstream no establece que ZZMI requiera exclusivamente Hook, y esa ausencia de requisito tampoco prueba que Direct Inject funcione con el juego.

El dominio `LoaderStrategyDecision` conserva `outcome = NoDecision`, sin modo preferido, y modela Hook como no aislado a proceso bajo la evidencia Wine consultada. Direct Inject queda como candidato por el resultado previo del Loader Lab, sin convertir ese test en compatibilidad de juego. `GameRuntimePlan.selected` permanece desconocido. La readiness resultante es `NeedsEvidence`; el gate no es elegible para revisión ni ejecuta procesos. La vista Advanced resume estas distinciones y deja criterios, capabilities y rutas técnicas en detalles expandibles.

En 0.8.1 no se ejecutó Hook, Direct Inject, Steam, Proton, Wine o ZZZ; tampoco se modificó Steam, el juego, compatdata ni prefix. Fuentes y límites: `../../01_Descubrimiento/estrategia_loader_zzmi_linux.md`, [`ADR-027`](../../04_Arquitectura_y_seguridad/ADR/027_xxmi_loader_strategy_for_lxmi.md) y [`verificación 0.8`](../../06_Verificacion/verificacion_0_8.md).

## LXMI-0.8.2 — contexto Steam / Proton

`lxmi-steam` parsea en modo read-only `InstallConfigStore/Software/Valve/Steam/CompatToolMapping` desde el `config/config.vdf` del Steam root asociado a la biblioteca del juego. El lector limita el tamaño, valida archivo/directorio regulares y rechaza symlinks. Un mapping por AppID, el mapping global `0` y la lista de compatibility tools instaladas se presentan por separado.

El servicio agrega esos datos a `SteamProtonProcessContextEvidence` junto con AppID, biblioteca, manifest, compatdata/pfx candidatos y findings con fuentes. Una configuración presente no se eleva a runtime efectivo; el runtime, prefix y wineserver efectivos siguen `Unknown` sin observar una ejecución. Readiness y el gate permanecen `NeedsEvidence`/no elegible, `LoaderStrategyDecision` sigue `NoDecision`, y el guard Hook no cambia.

En el host consultado el mapping global y por AppID no estaban presentes en la sección legible de `config.vdf`; dos `localconfig.vdf` revisados no contenían `CompatToolMapping`. Se observaron el manifest/AppID 4162040, ejecutable y directorios `compatdata/4162040/pfx`. No se ejecutó ningún proceso externo ni se modificó Steam, el juego, compatdata o prefix. Ver [investigación local/upstream](../../01_Descubrimiento/evidencia_contexto_steam_proton_0_8_2.md) y [verificación 0.8.2](../../06_Verificacion/verificacion_0_8_2.md).

## LXMI-0.8.3 — observación pasiva (cerrada)

El árbol de trabajo incorpora un observer efímero de Linux procfs y una sección Advanced para observar metadatos de una ejecución normal de ZZZ iniciada manualmente por el usuario desde Steam. El backend deriva el objetivo desde el manifest de AppID `4162040`, filtra procesos por UID actual, lee solo claves environment allowlisted, limita la sesión a 90 segundos y no ofrece señales/control. El snapshot se presenta separado de mapping configurado y Proton candidates; no modifica `GameRuntimePlan.selected`, estrategia loader o el gate.

La observación manual en la ventana Tauri mostró un resultado real. La sesión comenzó con un primer scan `WaitingForGame` y, durante la ventana, aceptó un proceso `ZenlessZoneZero.exe` (PID `691954`). Se observó AppID `4162040`, Proton Experimental `1790244902` / `experimental-11.0-20260924-x86_64`, compatdata coincidente y `WINEPREFIX` observado. Un proceso wineserver del usuario mostró el mismo `WINEPREFIX`; esto no prueba que el juego use esa instancia. La lectura allowlist fue denegada en algún punto del contexto de proceso/ascendencia; el contador previo solo contaba candidatos directos, y la UI ahora distingue ambos alcances. La versión del producto se actualizó a `0.8.3`. El runtime observado describe esa sesión y no cambia la selección configurada ni `GameRuntimePlan.selected`. Diseño/evidencia: [observación 0.8.3](../../01_Descubrimiento/observacion_runtime_zzz_steam_0_8_3.md); verificación: [verificación 0.8.3](../../06_Verificacion/verificacion_0_8_3.md).

## LXMI-0.8.4 — Proton/Wine Context Lab

`lxmi-bridge/src/context_lab.rs` reutiliza el executor del bridge con peers fijos y hashes de build. Hay relaciones independientes para tool, compatdata, prefix, wineserver, mappings completos y mapping del marker. Un evento nombrado con nonce mide visibilidad de objetos; no enumera/abre procesos. GameContextJoinPlan no tiene executor.

Construcción Windows (MinGW GCC x86-64, sin sudo/install automático):

```bash
bash tools/lxmi-context-peers/build.sh
```

Desde Tauri:

1. Scan Steam.
2. Abrir Advanced / Proton-Wine Context Lab.
3. Consultar y elegir Proton explícito.
4. Consentir prefixes privados y probar ambos casos.

 El script Proton se valida contra el discovery; el frontend no puede enviar prefix, executable, DLL o PID. Si falta un artefacto o cambia su hash se presenta un error específico; no se ejecuta.

El staging no sobrescribe un artefacto anterior diferente: informa `HelperHashMismatch` para revisión. Los bins locales de desarrollo pueden cambiar de hash entre builds MinGW por su timestamp PE; una futura actualización de artefactos requerirá un procedimiento explícito. No se alteran paquetes oficiales ni el runtime ZZMI. Las sesiones usan directories nuevos por nonce y permiten inspección posterior; los prefixes de laboratorio son regenerables. No se elimina ni señaliza wineserver globalmente.

Los host tests de `lxmi-bridge/tests/context_host.rs` están ignorados normalmente. Para una ejecución explícita se requieren `LXMI_RUN_CONTEXT_HOST_TEST=YES`, `LXMI_HOST_MANAGED_ROOT`, `LXMI_CONTEXT_BUILD_ROOT`, `LXMI_HOST_PROTON_SCRIPT` y `LXMI_HOST_STEAM_ROOT`. `LXMI_CONTEXT_MODE=different` selecciona el control; sin esa variable usa mismo prefix. `cargo test -p lxmi-bridge --test context_host -- --ignored --nocapture` ejecuta únicamente el fixture elegido; no lo uses con el prefix del juego. [Resultados](../../06_Verificacion/verificacion_0_8_4.md).

## LXMI 0.8.5 — Safe target binding

Advanced → Context Lab mantiene casos same-prefix y cross-prefix separados. Target protocol 2 publica Windows PID, imagen de ruta única por nonce y creación FILETIME; controller correlaciona child propio y runner consulta solo ese PID con query limitada. Solo Verified habilita el Inject oficial sobre la DLL propia, sin pipe requerido entre prefixes.

Same-prefix: identidad Verified, Inject 0, marker PASS. Cross-prefix: PidNotVisible Win32 87, BLOCKED_NOT_ATTEMPTED. Filesystem visible no prueba proceso visible. Loader strategy NoDecision y game gate cerrado; no writes a Steam/ZZZ/4162040 ni Hook.

Los Windows fixtures se compilan con `tools/lxmi-loader-lab/build.sh`; tests normales offline y host opt-in separados. [Verificación 0.8.5](../../06_Verificacion/verificacion_0_8_5.md).

## LXMI 0.8.6 — Context Re-entry

Advanced Context Lab ofrece Run Same-Prefix Re-entry y Run Different-Prefix Control. Reutiliza target/executor con helper metadata-only compilado por tools/lxmi-loader-lab/build.sh. El helper no contiene ruta Inject y no requiere packages oficiales. Same-prefix late join Verified; control PrefixMismatch, evento invisible y PID collision. Prefix identity y endpoint candidate se muestran por separado; game entry sigue no probado y gate cerrado. [Verificación](../../06_Verificacion/verificacion_0_8_6.md).
