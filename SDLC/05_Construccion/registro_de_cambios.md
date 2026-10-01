# Registro de cambios

Las entradas 0.1 y 0.2 registran el estado de verificación de esos incrementos en su momento; las validaciones posteriores se indican en la entrada 0.3 y en la guía vigente.

## LXMI-0.1 — primer incremento técnico (2026-09-27)

### Implementado

- Workspace Rust con lxmi-core, lxmi-steam y el paquete desktop Tauri 2.
- UI React/TypeScript: información de Linux, acción Scan Steam, estados/avisos y Wuthering Waves como sin escanear.
- Servicio Rust de información OS/arquitectura/home/XDG.
- Detector de rutas Steam comunes, canonicalización/deduplicación, lectura de steamapps/libraryfolders.vdf y validación de bibliotecas.
- Parser VDF acotado con comentarios, objetos, valores escapados, formatos legado/actual y límites de tamaño/profundidad.
- Errores tipados y logging JSON con tracing.
- Pruebas con fixture y directorios temporales; no se accedió a una instalación Steam real.

### Verificado

- Pasaron cargo fmt --all -- --check; 16 tests de lxmi-core/lxmi-steam; y Clippy de ambos crates con `-D warnings`.
- `npm ci` instaló el lockfile sin vulnerabilidades reportadas. Pasaron typecheck, ESLint, Prettier y build Vite; el servidor Vite inició y respondió su documento HTML en loopback.
- cargo metadata --offline --no-deps reconoció los tres miembros del workspace.

### Pendiente/bloqueado

- El chequeo/arranque nativo de Tauri no se completó: faltan WebKitGTK 4.1/JavaScriptCoreGTK y libsoup 3 en el Ubuntu de trabajo. No se usó root ni se instalaron paquetes del sistema.
- No se ejecutó el test completo del workspace ni clippy del paquete Tauri.
- No se detectaron juegos, Proton/Wine o prefixes y no se integró XXMI.

## LXMI-0.2 — Steam Game + compatdata Discovery (2026-09-27)

### Implementado

- Reutilización del parser KeyValues/VDF existente para `appmanifest_*.acf`, sin dependencias nuevas; campos requeridos, AppID positivo, claves duplicadas, rutas seguras y errores tipados.
- Enumeración de manifests por bibliotecas Steam; archivos inválidos/malformados se reportan y no detienen el resto del escaneo. Se verifica que AppID del archivo y payload coincidan.
- Registro central de juegos soportados e identificación de Wuthering Waves por AppID `3513350`, verificado en la [ficha oficial de Steam](https://store.steampowered.com/app/3513350/Wuthering_Waves/). La ficha también menciona Anti-Cheat Expert a nivel de kernel; se registra como riesgo futuro, sin inferir compatibilidad ni integrar o eludir controles.
- Modelo genérico `Game`/`GameInstallation`, con estado de instalación y discovery de `compatdata/<AppID>/pfx` solo mediante metadata de lectura.
- Rechazo de symlinks en manifests, `steamapps/common`, carpeta de juego, compatdata y pfx. El UI muestra AppID, ruta esperada, estado de carpeta, compatdata/pfx candidato y avisos.
- Fixtures sintéticos de manifests y tests de registry, parser, scanner, rutas, resultados ausentes y symlinks.

### Verificado

- `cargo fmt --all`, 39 tests de `lxmi-core`/`lxmi-steam` y Clippy de ambos crates con `-D warnings` pasaron.
- `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build` pasaron.
- No se leyó Steam real y no se ejecutó ninguna operación de escritura sobre Steam, juegos o prefixes.

### Pendiente/bloqueado

- `cargo check --offline --locked -p lxmi-desktop` se detuvo en `soup3-sys`/`javascriptcore-rs-sys`: faltan libsoup 3 y JavaScriptCoreGTK 4.1. No se comprobó compilación del adapter Tauri ni se abrió la ventana.
- La AppID del producto fue comprobada en la ficha Steam; no se comprobó una instalación local ni soporte Proton/XXMI.

## LXMI-0.3 — Proton Runtime Discovery (2026-09-27)

### Implementado

- Nuevo crate `lxmi-proton`; reutiliza el parser KeyValues de `lxmi-steam` para leer `toolmanifest.vdf` y `compatibilitytool.vdf` sin parser duplicado.
- Descubrimiento de candidatos en `steamapps/common` de bibliotecas registradas y en `<Steam root>/compatibilitytools.d`, sin crear rutas.
- Clasificación de Proton por metadata layer `proton` y/o metadata custom acompañada de entrypoint `proton`; distingue Steam Linux Runtime mediante layer de runtime y metadata `VERSIONS.txt`, además de otras herramientas y tipo desconocido.
- Lectura acotada y opcional de `version`/`VERSIONS.txt`; version ausente no invalida un Proton estructuralmente válido.
- Validación de rutas declaradas, traversal y symlinks; solo se inspecciona existencia/tipo del entrypoint, no se ejecuta.
- Command Tauri agrega el resultado; UI muestra herramientas, source, tipo, versión, ID, rutas, estado e incidencias. Para Wuthering Waves mantiene `Proton seleccionado: No determinado`.
- Footer/copy de UI deja claro que el escaneo es de lectura y no ejecuta Proton/Wine ni modifica Steam, juegos o prefixes.

### Verificado

- Fixtures cubren metadata Proton/custom, Steam Linux Runtime, layouts incompletos, sin versión, malformados, paths absolutos documentados, traversal y symlinks.
- Escaneo local read-only: una Steam Installation/Library, Wuthering Waves AppID `3513350` con directorio presente y compatdata ausente; detectó Proton Experimental y tres Steam Linux Runtime con versiones disponibles. Una entrada symlink `Steam.dll` fue reportada y omitida.
- Pasaron `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` (57 tests), `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`.
- El workspace Tauri compila con dependencias nativas disponibles y la ventana se inició/revisó visualmente con los resultados del scan. No se probó una custom tool real, selección de Proton por juego, ejecución de juego/runtime ni compatibilidad XXMI.
- Sin modificaciones a Steam, juegos, Proton, launch options, environment variables, compatdata o pfx.

## LXMI-0.4 — Runtime Planning & Launch Readiness (2026-09-27)

### Implementado

- Nuevo crate `lxmi-runtime`: compone descubrimiento de juego, compatdata/pfx y compatibility tools en un plan declarativo y efímero por juego.
- El modelo distingue instalación detectada/no encontrada/desconocida, completitud desconocida, política runtime, selección, candidatos, estados de compatdata/prefix, requisitos, evidencia, issues y readiness.
- Solo tools clasificadas como Proton y estructuralmente válidas se exponen como candidatos. Steam Linux Runtime, tools desconocidas e instalaciones incompletas no se promueven a Proton.
- La selección permanece desconocida sin evidencia fiable por juego, incluso con un único candidato. `runtime discovery != runtime selection`; `runtime selection != launching`.
- Tauri serializa el plan; la UI funcional muestra readiness, runtime seleccionado/no determinado, candidatos, prefix y evidencia expandible. Versión del desktop alineada a `0.4.0`.
- `Steam.dll` en `steamapps/common` no es candidato a tool. Ese nombre concreto se omite sin seguir el symlink y se reporta como observación informativa, sin volver parcial el discovery. Los demás symlinks candidatos siguen como advertencia y los destinos no se siguen.
- El readiness evalúa solo la base observada para una preparación futura. No confirma lanzamiento, compatibilidad, salud del prefix ni disponibilidad de XXMI/WWMI.

### Verificado

- `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` y `cargo test --workspace`: pasaron; 69 tests en total, incluidos 10 tests sintéticos del planner y una regresión de symlink.
- `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`: pasaron.
- `npm run tauri:dev`: la ventana nativa inició. Scan local read-only: Wuthering Waves, AppID `3513350`, directorio presente; compatdata/pfx ausentes; Proton Experimental y Proton Hotfix como dos candidatos; tres Steam Linux Runtime; sin tool custom detectada. Selección `Unknown`; readiness `NeedsInitialization`; discovery completo con una observación informativa por `Steam.dll`.
- Los datos de Planner tests son `MOCK / FIXTURE`; la instalación y los runtimes enumerados en el scan son observaciones locales `COMPROBADO`. No se ejecutó Steam, Proton, Wine ni el juego. No hubo escritura sobre Steam, juegos, launch options, environment variables, compatdata o pfx.
- No se probó una compatibility tool custom real ni una selección forzada por juego.

## 0.5 — XXMI / WWMI Integration Foundation

### Implementado

- `crates/lxmi-xxmi`: modelos separados de paquete WWMI y XXMI Libraries, detección read-only acotada en ubicaciones conocidas y evidence estructural. Falta de paquete es estado normal; DLL aislada no valida runtime.
- Importación explícita de directorios locales solamente. Inventario y SHA-256; metadata `lxmi-package.json` de LXMI queda fuera del payload. Versiones se conservan como texto raw.
- Staging privado bajo XDG, límites 128 MiB/archivo, 512 MiB total, 4096 entradas, 24 niveles; rechazo de traversal, absolute paths, symlink, hardlink, special files y colisiones de case; promoción Linux `renameat2(RENAME_NOREPLACE)`.
- Revalidación de bytes/manifiesto al listar y antes del plan. Autenticidad de firma no verificada.
- Assessment WWMI y requisitos separados; runtime selección queda la del planner (Unknown sin evidencia). Compatibilidad de lanzamiento no verificada.
- Plan revisable propone almacenamiento administrado fuera del juego, fuentes, hashes, Create/Unchanged/ReplaceWithBackup y backup requerido. No hay executor/apply y 0.5 no escribe Steam/juego/prefix.
- UI Tauri con inspección, importación local y selección de paquetes para revisión. El prototipo visual sigue separado.
- Auditoría de upstream, releases y licencias en `SDLC/01_Descubrimiento/ecosistema_xxmi_wwmi.md`; ADR-019, ADR-020 y ADR-021.

### Comprobado

- Releases oficiales WWMI-PACKAGE v1.0.0 y XXMI-PACKAGE v1.1.7 contrastadas sin ejecutar binarios; hash SHA-256 anotado en investigación. Validadas/importadas desde carpetas extraídas en `/tmp`; plan generado en storage temporal. Son paquetes upstream reales, no fixtures.
- Suite usa fixtures sintéticos para payloads de texto: 45 nuevos; workspace 114 tests.
- `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy con `-D warnings`, `cargo test --workspace`, frontend typecheck/lint/format/build pasan. `npm run tauri:dev` inicia v0.5.0.
- Scan Steam local read-only del 2026-09-27: Wuthering Waves directory presente, ejecutable Win64 candidato ausente, compatdata/pfx ausentes, Proton selection Unknown, readiness `NeedsInitialization`; no se detectó WWMI en ubicaciones comprobadas.
- Nuevo scan local read-only del 2026-09-28: Wuthering Waves no está en los manifests de la biblioteca detectada; directorios Proton vistos no tienen metadata/entrypoint válidos; compatdata ausente y readiness `Blocked`. Estado actual, distinto de la observación histórica de 0.4.
- Captura confirmó ventana inicial; navegador separado confirmó controles/labels del panel sin overflow. El flujo Tauri IPC de importación no se recorrió clic a clic: **NO COMPROBADO**.

### Límites

- Sin extracción ZIP, descarga, apply, backups reales, launch options, D3D overrides o instalación de runtime en el juego.
- SHA-256 comprueba consistencia con el import, no autentica origen. Manifiesto upstream de XXMI Libraries declara firmas, LXMI no las verifica.
- Compatibilidad bajo Linux/Proton y permisos para redistribuir el conjunto completo de binarios/recursos **NO COMPROBADOS**.
- No se ha hecho power-loss fault injection. El atomic rename no equivale a transacción durable de 0.6.

## LXMI-0.5.1 — Zenless Zone Zero / ZZMI Integration Validation (2026-09-28)

### Implementado

- `lxmi-core` agrega Zenless Zone Zero al registro por Steam AppID `4162040`, con ejecutables esperados `ZenlessZoneZero.exe` y `ZenlessZoneZeroBeta.exe`; Wuthering Waves se conserva.
- `lxmi-steam` reconoce manifests de ZZZ, deriva la ruta desde el `installdir` del manifest, busca el ejecutable con límite de profundidad/entradas y consulta `compatdata/<AppID>/pfx` sin escribir ni seguir symlinks.
- `lxmi-runtime` y el DTO Tauri transportan distribución, ruta/estado de ejecutable y compatdata/prefix para el juego detectado. La selección Proton sigue desconocida aunque se encuentre un candidato.
- `lxmi-xxmi` registra ZZMI como integración de Zenless Zone Zero, conserva la validación WWMI, rechaza la asociación cruzada, lee versión raw de `Core/ZZMI/main.ini`, modela XXMI Libraries como dependencia separada y deja Steam/Linux-Proton explícitamente sin verificar.
- El import local ZZMI usa el pipeline existente de carpeta → staging → inspección → SHA-256 → managed storage; no se añadió descarga ni extracción de archives.
- El plan ZZMI es declarativo y el destino propuesto permanece dentro del almacenamiento administrado por LXMI. No incluye deploy al juego, cambios de settings, Steam o prefix, ni executor/apply.
- La UI funcional permite seleccionar ZZZ/ZZMI y muestra advertencias de compatibilidad no verificada. No se fusionó el prototipo visual.
- Se alinea la versión visible, el package desktop, Tauri y `lxmi-xxmi` a `0.5.1`.
- Se añade la investigación upstream ZZMI/XXMI Libraries, licencias consultadas y checklist manual no destructiva.

### Upstream verified

- La configuración upstream del XXMI Launcher declara los ejecutables ZZZ, la carpeta `ZenlessZoneZero Game`, `importer_folder='ZZMI/'`, el paquete `leotorrez/ZZMI-Package` y `requirements=['XXMI']`.
- El `d3dx.ini` de ZZMI selecciona `ZenlessZoneZero.exe`, incluye `Core/ZZMI/main.ini` y avisa que la release se entrega para XXMI Launcher y no incluye las DLL requeridas de 3Dmigoto.
- `Core/ZZMI/main.ini` incluye `Libraries/Includes.ini`, `d3dx_patch.ini` y `help.ini`; la versión local que LXMI registra como raw se declara allí. El contrato estructural acotado no equivale a inventariar/authenticar un release.
- El upstream consultado documenta metadata de firma para la release, y la configuración del launcher declara patrón y clave pública. LXMI no implementa esa verificación: SHA-256 local no es autenticidad.
- El repositorio ZZMI declara GPL-3.0; el launcher publica GPL-3.0; los componentes/recursos redistribuibles de XXMI Libraries requieren auditoría de licencia individual. LXMI no redistribuye packages upstream.
- El issue upstream de soporte ZZZ Steam y petición adicional Linux/Proton seguía abierto en la fecha de consulta; LXMI conserva ambos estados como `Unverified`.

Fuentes y límites completos: `SDLC/01_Descubrimiento/ecosistema_xxmi_zzmi.md`.

### Comprobado

- **Host local read-only:** el manifest Steam local confirma AppID `4162040`; directorio derivado de manifest presente; ejecutable `ZenlessZoneZero.exe` encontrado bajo `games/ZenlessZoneZero Game/`; `compatdata/4162040/pfx` presente como candidato.
- El test host-only del snapshot usado por el command Tauri reportó distribución Steam, instalación/exe presentes, `prefix_found`, un candidato Proton, selección `unknown` y readiness `incomplete`. La suite estándar no depende de que ZZZ esté instalada; esta prueba está marcada `#[ignore]`.
- La UI Tauri dev se compiló e inició. El clic manual del botón `Scan Steam` en la ventana y el panel ZZMI mediante interacción nativa quedaron **NO COMPROBADOS**.
- Las pruebas sintéticas cubren el manifest/ejecutable ZZZ, package ZZMI válido/incompleto, dependencia XXMI Libraries, cruces WWMI↔ZZMI, plan sin target de juego, falta de firma/autenticidad y selección Proton sin inferencia.
- No se descargó/importó release ZZMI real, no se probó ZZMI + Steam/Linux/Proton, no se lanzó el juego y no se hizo apply. No se modificaron Steam, ZZZ, compatdata o pfx.

### Gates finales

- `git diff --check`, `cargo fmt --all -- --check`, `cargo check --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`: pasaron.
- `cargo test --workspace`: 126 pasaron y 1 test host-only quedó ignorado por defecto. El host-only se ejecutó aparte y pasó leyendo Steam/ZZZ localmente.
- `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`: pasaron.
- `npm run tauri:dev`: compiló y lanzó el binario desktop. La interacción manual en la ventana no se verificó.
- Este incremento no confirma compatibilidad de ZZMI con Steam, Linux o Proton.

La evidencia detallada está en `SDLC/06_Verificacion/verificacion_0_5_1.md`.

## LXMI-0.5.2 — Official Package Acquisition, Authenticity & Install Mapping (2026-09-28)

### Implementado

- Adaptador de releases GitHub limitado a los repositorios oficiales ZZMI y XXMI Libraries. Conserva release ID/tag/commit/asset, SHA-256 de GitHub, timestamps y el archivo companion `Manifest.json`; consulta solo al iniciar la acción del usuario.
- Descarga HTTPS con hosts permitidos, redirects acotados, límites de bytes/tiempo, streaming a cache XDG `.partial`, hash y limpieza en error.
- Verificación upstream reproducida desde el XXMI Launcher fijado: ECDSA P-384 sobre SHA-256, firma Base64 ASN.1 DER aplicada a los bytes ZIP. `Manifest.json` de Libraries se coteja por SHA-256 publicado; además se verifican las firmas upstream de `3dmloader.dll`, `d3d11.dll` y `d3dcompiler_47.dll`.
- Inspección/extracción ZIP con staging privado, límites de archivo/entrada/expansión y rechazo de traversal, rutas absolutas, colisiones normalizadas, enlaces y archivos especiales; inventario con SHA-256 y promoción a storage XDG.
- Paquetes oficiales ZZMI y XXMI Libraries permanecen separados; la dependencia se resuelve cuando ambos están gestionados. La metadata de provenance está en `lxmi-package.json` fuera del payload.
- Mapeo declarativo derivado de la configuración upstream, inventario de release y hash-read dry-run de la carpeta candidata del ejecutable. El `importer_path` configurable de XXMI no se conoce aún; raíz configurada permanece `null`, destino candidato no autoritativo, `apply_allowed=false` y `writes_performed=false`.
- Panel Tauri consulta y presenta releases pinneadas, descarga/verifica/importa cada paquete, estado de dependencia y dry-run. No se integró el prototipo visual.
- Investigación fijada en `SDLC/01_Descubrimiento/adquisicion_paquetes_xxmi_0_5_2.md`; ADR-024 registra confianza por fuente; la verificación está en `SDLC/06_Verificacion/verificacion_0_5_2.md`.

### Verificado

- **UPSTREAM VERIFIED:** tags, release IDs, assets, commits resueltos, fecha, SHA-256 GitHub, companion manifest y claves se consultaron en las fuentes oficiales indicadas en la investigación.
- **UPSTREAM VERIFIED:** en la consulta explícita de LXMI se seleccionaron ZZMI v1.5.0 (release `393483881`, commit `e59f87047cd405c5db5476d3b1b499574bc43d67`, asset `580682882`, publicado 2026-09-22T05:54:50Z) y XXMI Libraries v1.1.7 (release `387957029`, commit `6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9`). La v1.4.5 inicial permanece como versión histórica administrada; no se presenta como la selección vigente.
- **CRYPTOGRAPHICALLY VERIFIED:** ZIP de ZZMI v1.5.0 y Libraries v1.1.7 pasaron la firma P-384/SHA-256. El Manifest de Libraries declaró tres firmas DLL, todas verificadas con la clave upstream.
- **COMPROBADO:** el cliente LXMI descargó por HTTPS los dos paquetes y el Manifest en cache/store temporales mediante prueba explícita de red. La UI también descargó/verificó/importó los paquetes seleccionados a storage XDG. El inventory contiene ZZMI v1.5.0 (47 archivos, 1,116,062 B), v1.4.5 histórica (33 archivos, 1,041,827 B) y Libraries v1.1.7 (4 archivos, 8,096,499 B).
- El dry-run sobre ZZZ real combinó v1.5.0 + Libraries v1.1.7 y comparó 49 mappings contra la carpeta candidata del ejecutable: 49 `WouldCreate`; el snapshot de cada destino candidato quedó igual antes y después. El candidato no es el `importer_path` configurado y no es destino aprobado.
- Suite offline del crate y los quality gates completos se registran en `verificacion_0_5_2.md`. La prueba real upstream está marcada `#[ignore]`; la suite normal no accede a Internet.
- La ventana Tauri inició; desde ella se escaneó Steam/ZZZ, se consultaron releases, se descargaron/importaron los dos paquetes, se confirmó el set de dependencia y se generó el plan de 49 destinos. La UI mostró Steam/Linux/Proton `unverified`, apply no permitido y ninguna escritura.
- No se escribió en Steam, ZZZ, `compatdata` o prefix, y no se ejecutó contenido upstream.

### Límites

- **NO COMPROBADO:** compatibilidad de ZZMI con Steam, Linux/Proton, inyección o ejecución con mods; no se lanzó ZZZ.
- **NO RESUELTO:** la carpeta efectiva de `importer_path` se configura en XXMI y no está registrada por LXMI; el directorio del ejecutable es solo candidato de comparación.
- La autenticidad respecto a la clave fijada no implica que el publisher sea el mismo que el maintainer de cada componente; licencias de Libraries requieren revisión por componente. LXMI no redistribuye estos archivos.
- No existe apply, rollback, ni escritura al juego.

## LXMI-0.5.3 — Managed Importer Runtime & Launch Topology (2026-09-28)

### Implementado

- `lxmi-xxmi` ahora resuelve el importer de ZZMI como `App.Root/ZZMI/`, fuera de la carpeta del juego. El mapping del dry-run 0.5.2 contra la carpeta vecina al executable se conserva solo como comparación histórica, no como target aprobado.
- `RuntimeAssemblyPlan` compone packages oficiales autenticados ZZMI + XXMI Libraries. `assemble_zzmi_runtime` copia a staging privado, revalida hashes, deriva `d3dx.ini` cambiando solo `[Loader] target`, preserva `loader = XXMI Launcher.exe`, crea `Mods/` bajo LXMI y promueve a un root versionado bajo XDG. Los packages originales quedan separados e inmutables.
- Se ensamblan bajo `ZZMI/` los archivos ZZMI y las DLL que el código upstream de XXMI Launcher despliega en `importer_path` (`d3d11.dll`, `d3dcompiler_47.dll`). `3dmloader.dll` permanece en su package XXMI Libraries autenticado y se referencia mediante ruta/hash; no se ejecuta ni se designa como helper.
- `runtime-manifest.json` describe provenance, versiones, hashes y configuración fuera del payload upstream. El root administrado pertenece a LXMI y la operación se bloquea si solapa Steam, ZZZ, compatdata o prefix.
- `topology.rs` inspecciona `pfx/dosdevices` de forma no recursiva, registra symlinks/targets y calcula mappings Linux→Windows con validación de componentes. El resultado es evidence de filesystem, no prueba de carga del runtime.
- `LaunchTopologyPlan` combina ZZZ, compatdata/prefix, Proton candidates, runtime gestionado, mappings y blockers. La selección de Proton, loader/helper y requisito de mismo prefix quedan `Unknown`; ejecución sigue deshabilitada.
- UI Tauri añade revisar composición, preparar el runtime XDG e inspeccionar topología. No se fusiona el prototipo visual ni se habilita `apply`.
- Se registra la investigación fijada de upstream y ADR-025. La próxima fase se redefine como experimento de topología/bridge, no como instalador.

### Upstream verified

- XXMI Launcher v2.2.1, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`: `importer_path` relativo se resuelve desde `App.Root`; ZZMI fija `importer_folder = "ZZMI/"`. La semántica upstream de `loader` es identidad de proceso Windows en la configuración Loader, no nombre de proceso Linux LXMI.
- El código del paquete Migoto pasa `3dmloader.dll` al API `DllInjector` y coloca `d3d11.dll`/`d3dcompiler_47.dll` en el importer. `3dmloader.exe` es una ruta/herramienta mencionada por release notes para inyección directa y no se asume parte del runtime mínimo actual.
- Upstream documenta XXMI Launcher Portable para Linux bajo Wine 9.22+ y Microsoft Visual C++ Redistributable; eso no demuestra ZZZ Steam + Proton + ZZMI.
- Repositorios/versiones/archivos y límites de licencia están fijados en `../01_Descubrimiento/topologia_runtime_xxmi_linux.md`. La redistribución de cada binario sigue pendiente de revisar individualmente.

### Comprobado

- Quality gates Rust: `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy de workspace con `-D warnings`, `cargo test --workspace`: pasaron; 149 tests, 5 ignorados por ser explícitos/host-only/network.
- Quality gates frontend: `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm run build`: pasaron.
- Prueba opt-in `assembles_verified_zzmi_runtime_and_inspects_local_zzz_topology_without_game_writes`: 1 pasó con packages oficiales administrados ZZMI v1.5.0 y Libraries v1.1.7. Ensambló 49 archivos bajo `/home/university/.local/share/lxmi/runtimes/zenless-zone-zero/zzmi/`; el importer se mapeó por `Z:`. Selección de Proton `unknown`, `SamePrefixRequirement::Unknown`, `execution_enabled=false`, `external_files_modified=false`.
- Antes/después quedaron iguales las rutas candidatas `d3d11.dll`, `d3dcompiler_47.dll`, `d3dx.ini`, `ZZMI` bajo el directorio comparativo del executable, y el reporte de symlinks `pfx/dosdevices`. Los hashes/manifests de ambos packages fuente volvieron a validarse.
- **NO COMPROBADO:** no se hizo click a click en la ventana Tauri. Ya había una sesión `tauri dev` y una ventana LXMI; el browser externo renderizó el frontend sin bridge Tauri y no sirve como verificación funcional de sus commands. No se ejecutó ZZZ, Steam, Proton, Wine, launcher/helper, DLL, ni contenido del package.

### Límites

- El runtime ensamblado es administrado, pero no se ha probado que pueda lanzarse/cargarse bajo Proton.
- No se eligió estrategia entre launcher portable, componente upstream o helper propio; no hay autorización/licencia resuelta para redistribuir un helper.
- No se conoce qué proceso Windows satisface la identidad `XXMI Launcher.exe`, si debe compartir prefix con ZZZ, ni qué Proton seleccionará Steam.
- No se modificaron Steam, ZZZ, compatdata o prefix. No hay executor, inyección, anti-cheat bypass, launch options, `apply` ni rollback.

Detalles reproducibles en `../06_Verificacion/verificacion_0_5_3.md`.

## LXMI-0.6 — Controlled Windows Runtime Bridge Prototype (2026-09-28)

### Implementado

- Crates lxmi-bridge y lxmi-bridge-protocol, más el binario tools/lxmi-bridge-helper orientado a Windows.
- Protocolo stdin/stdout JSON versión 1, nonce aleatorio de 128 bits, versión/target/hash del helper y chequeo hash del runtime marker en ambos lados.
- Helper que solo deriva root desde helpers/, rechaza traversal/symlinks, lee un runtime-manifest.json acotado y devuelve JSON. No enumera procesos ni contiene loader/injector.
- Servicio Tauri avanzado que re-escanea Proton candidates, valida los IDs de packages y resuelve helper/runtime del storage LXMI; la UI no puede escoger rutas arbitrarias ni cambiar GameRuntimePlan.selection.
- Executor de Proton por argv directo, con environment limpiado/allowlisted, timeout 20 s, límite de salida 1 MiB por stream y terminación de su propio grupo.
- Prefix test exclusivo bajo XDG; el executor rechaza el modo de compatdata de juego. La UI exige elegir Proton explícitamente y aceptar sus posibles efectos en el test-prefix y la distribución Proton.
- Script de build/staging del helper Windows. Se instaló el target Rust Windows; no se instalaron paquetes del sistema.

### UPSTREAM VERIFIED

- Se fijó ValveSoftware/Proton tag proton-11.0-2 (commit corto db9e6ff) y se revisó README, entrypoint proton y release. La inspección del Proton local añadió evidencia de que `run` inicia `steam.exe`, mientras `runinprefix` ejecuta el helper directamente bajo Wine; LXMI usa el segundo. El script consume STEAM_COMPAT_DATA_PATH y STEAM_COMPAT_CLIENT_INSTALL_PATH, puede inicializar/actualizar pfx y contiene mantenimiento de dist/fixups. Referencias: ../01_Descubrimiento/bridge_linux_windows.md.
- Ese código upstream no garantiza API estable para ejecutar juegos fuera de Steam. Esta fase no extrapola sus hallazgos a ZZMI/ZZZ.

### Estado previo al cierre de host (histórico)

- La suite focal de las crates bridge/protocol/helper pasó: 17 tests de bridge/helper y 0 tests definidos en protocol.
- En la primera verificación pasaron los gates offline. En el cierre final se repitieron y `cargo test --workspace` reportó 166 pasados y 6 ignorados host/network/opt-in.
- En `apps/lxmi-desktop/` pasaron `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`.
- En ese punto, `cargo check` del helper target Windows pasó pero todavía no había un ejecutable enlazado.
- `npm run tauri:dev` compiló y lanzó la aplicación; en ese punto la interacción nativa aún no se había realizado.
- `bash -n tools/install-bridge-helper.sh` y `git diff --check` pasaron.

### Bloqueos iniciales (resueltos en el cierre siguiente)

- Se intentó cross-compilar con Zig 0.15.2: compiló crates/dependencias, pero el linker no encontró `msvcrt`; no se produjo ni staged el helper `.exe`. No se instaló MinGW ni paquetes del sistema. No se ejecutó Proton, Wine ni el helper.
- Host handshake, environment real, visibilidad/path roundtrip Windows y prefix generado por Proton: **NO COMPROBADOS**.
- No se hizo click a click en la UI nativa; no se modificaron ZZZ, Steam, compatdata de juego ni prefix.
- LXMI 0.6 no se considera cerrado ni habilita el inicio de 0.7 hasta compilar/stagear el helper y obtener éxito en el host bridge test aislado.

### Cierre de validación host (2026-09-28)

- Se enlazó `lxmi-bridge-helper.exe` para `x86_64-pc-windows-gnu` usando `x86_64-w64-mingw32-gcc` GCC 13-win32; `file` confirmó PE32+ x86-64, 1,517,755 bytes. El SHA-256 del build y del helper staged coincide: `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2`. El fallo previo de Zig/`msvcrt` queda resuelto; no se instalaron paquetes con sudo.
- **HOST TEST positivo:** Proton Experimental `experimental-11.0-20260924-x86_64`, seleccionado solo para bridge test, ejecutó `runinprefix`; protocolo 1/nonce correctos, helper v0.6.0, exit 0, helper leyó el runtime marker mediante ruta Windows Z: y el SHA-256 del manifiesto coincidió (`c86d41865cc81c7639eecc17f021d80448fd2e12900453cc66f37d00c5332a5e`). La selección de runtime del juego permaneció `unknown`.
- **HOST TEST negativo:** runtime-relative path inexistente retornó respuesta JSON correlacionada con `success=false`, `runtime_not_visible` y exit 2; sin panic ni timeout.
- Se recorrió el flujo desde la ventana Tauri nativa: Steam/ZZZ scan, paquetes, selección explícita, aceptación de warning y resultado de bridge con hashes visibles. También se observaron helper ausente (restaurado después), runtime administrado no disponible y, en un HOME/XDG aislado, el estado sin candidato Proton.
- La inspección nativa del estado helper ausente encontró un párrafo HTML anidado; se corrigió el JSX y se volvió a probar sin warnings React/Vite. Typecheck, ESLint, Prettier y build pasaron tras el arreglo.
- Proton escribió en el contexto privado `$XDG_DATA_HOME/lxmi/test-prefixes/bridge-v1/` (incluido su CompatData aislado) y tocó el timestamp del `dist.lock` de su propia distribución. No usó `compatdata/4162040`; el executable y targets vigilados de ZZZ permanecieron iguales/ausentes.
- La tabla y clasificación final, incluyendo todos los quality gates, están en `SDLC/06_Verificacion/verificacion_0_6.md`. LXMI 0.6 queda cerrado; LXMI 0.7 no se inició.

## LXMI-0.7 — Upstream Loader Compatibility Experiment (2026-09-28)

### Implementado

- Investigación fijada a XXMI Launcher v2.2.1 commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`, release notes v2.1.5 y XXMI Libraries v1.1.7 commit `6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9`. Se documenta la diferencia: Hook global es el default observado de ZZMI; el test escogió la ruta Direct Inject como experimento aislado. `3dmloader.exe` no se ejecuta.
- `lxmi-bridge::LoaderExperimentPlan` describe Proton explícito, prefix, target, DLL, loader, marker y timeout. Un `ValidatedLoaderExperimentPlan` privado se construye solo después de la verificación de storage, provenance/hash y rutas fijas; de él se obtiene el argv de `runinprefix`. No se aceptan targets/PIDs/DLL desde UI.
- Se agregaron runner, proceso target y DLL de prueba propios para Windows x86-64. El runner crea su único target con `CreateProcessW` y pasa únicamente ese PID al export `Inject` upstream. La DLL verifica proceso anfitrión y nonce y crea marker JSON controlado. No se accede a otros procesos.
- Staging exclusivo bajo `$XDG_DATA_HOME/lxmi/tests/loader-v1/`; el `3dmloader.dll` se valida contra package Libraries v1.1.7 firmado y hash fijado, no se copia al repo ni se redistribuye. Proton opera solo bajo el test prefix `loader-v1` después de confirmación explícita.
- Advanced UI añade inspección, staging, runtime de prueba explícito, modos cerrados y resultado del plan validado. La versión de desktop pasa a `0.7.0`.
- ADR-027 registra la decisión limitada al laboratorio; `loader_xxmi.md`, arquitectura, seguridad y `verificacion_0_7.md` contienen fuentes, licencia, hashes, pruebas y unknowns.

### Verificado

- **UPSTREAM VERIFIED:** ruta actual de ZZMI y API de `DllInjector` revisadas con el código fijado. La release v2.1.5 menciona un injector custom en `3dmloader.exe` para Direct Inject; no se extrapola a default ZZMI.
- **HOST TEST:** MinGW compiló los 3 artefactos propios; el loader upstream compilado/staged se validó por hash.
- **HOST TEST:** Proton Experimental ejecutó baseline, Direct Inject positivo, target ausente, DLL ausente y nonce incorrecto con resultados/exit codes esperados. El test positivo verificó DLL externa, marker, nonce, identidad y mapping Windows real.
- **HOST TEST Tauri nativo:** se inspeccionó Loader Lab, seleccionó runtime de test, aceptó el prefix aislado y recorrió baseline, positivo y target ausente desde la ventana nativa.
- **COMPROBADO:** snapshots/assertions de ZZZ y `compatdata/4162040` no cambiaron. Proton usó el prefix de prueba LXMI y pudo modificar su propia distribución (`dist.lock`).
- La selección Proton del juego continúa `Unknown`. No hay test de Hook default, ZZZ/ZZMI, Steam/Linux/Proton del juego, `3dmloader.exe` o anti-cheat.

### Validación final

Pasaron `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy con `-D warnings` y `cargo test --workspace` (174 pasados, 7 ignorados opt-in/host/network). Los tres artefactos Windows de Loader Lab y el helper de 0.6 compilaron; typecheck, ESLint, Prettier, Vite build y `git diff --check` pasaron. El host test Proton pasó en los cinco escenarios. La tabla de comandos, hashes y límites está en `SDLC/06_Verificacion/verificacion_0_7.md`.

## LXMI-0.8 — Hook Mode Parity & ZZZ Launch Readiness (2026-09-28)

- Fijada de nuevo la investigación a XXMI Launcher v2.2.1 (`d56786b8dacb00c35204bff45ff5b8b83bd8962a`) y ZZMI v1.5.0. La ruta default de ZZMI es Hook; upstream lo prepara antes del target, espera/verifica el proceso y después limpia con `UnhookLibrary`.
- Documentada la semántica de `d3dx.ini`: `target` nombra el proceso destino y `loader` la identidad permitida para cargar el módulo desde una ubicación externa. El nombre de target no se entrega al export upstream `HookLibrary`, por lo que la configuración no prueba aislamiento single-target.
- Añadido `Hook` al enum cerrado del Loader Lab con rechazo backend `HookScopeNotIsolated` antes de validar paths, Proton o procesos. No se ejecuta el Hook global; existe test del fail-closed sin I/O.
- Añadido `ZzmiLaunchReadiness` en `lxmi-xxmi` con requisitos, capabilities, blockers/unknowns y gate declarativo. Proton de laboratorio permanece separado de runtime seleccionado para el juego; el gate no ejecuta ZZZ.
- Añadidas vistas Advanced para explicar el bloqueo Hook y evaluar readiness. Aplicación versionada `0.8.0`.
- **Resultado:** Hook `BLOCKED` para el laboratorio y ejecución/remote DLL Hook `NO COMPROBADOS`. El readiness no autoriza LXMI 0.9.
- Actualizada investigación, ADR-027, arquitectura y seguridad; añadido `01_Descubrimiento/launch_zzmi_steam_linux.md` y reporte `06_Verificacion/verificacion_0_8.md`.

## LXMI-0.8.1 — Loader Strategy Resolution for ZZMI on Linux (2026-09-29)

### Implementado

- Fijada la investigación a XXMI Launcher `v2.2.1`, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`, y trazados `ModelImporterPackage.use_hook`, `ZZMIPackage`, `StartGame`, `MigotoManager.StartAndInject` y `custom_launch_inject_mode`.
- Registrado que ZZMI hereda Hook como default; Custom Launch permite rutas Hook, Inject y Bypass. Inject usa la rama genérica `3dmloader.dll`; no se concluye compatibilidad de esa ruta con ZZMI/ZZZ Linux/Proton. `loader=` sigue siendo configuración de identidad de carga externa, no selector del modo.
- Comparado el cambio de default de WWMI como precedente arquitectónico solamente. Hook queda `NotEstablished` como requisito exclusivo; esto no se interpreta como `HookNotRequired`.
- Modelados `LoaderStrategyDecision`, candidatos vs. modo preferido, criterios, evidencia de aislamiento y componentes independientes del gate. El guard de Hook Lab sigue rechazando Hook antes de filesystem/process work.
- Recalculada readiness sin inferir selección Proton del juego: estrategia `NoDecision`, Proton seleccionado `Unknown`, readiness `NeedsEvidence`, gate no elegible para revisión.
- Advanced → ZZZ Readiness presenta el resumen de Hook, Direct Inject de fixture y compatibilidad de juego por separado; los criterios/rutas técnicas quedan en detalles.
- Actualizados investigación, ADR-027, arquitectura, modelo de datos, seguridad, backlog y guías de verificación/aplicación. Versión de la app incrementada a `0.8.1`.
- Quality gates finales: Rust workspace format/check/clippy/test **PASS** (186 tests passed; 7 host/network/opt-in ignorados), frontend typecheck/lint/format/build **PASS**, `git diff --check` **PASS**. La ventana Tauri nativa confirmó scan de ZZZ, packages ZZMI/XXMI Libraries managed, runtime íntegro y readiness `NeedsEvidence`; Proton seleccionado para el juego `Unknown`. No se ejecutaron modos de loader ni runtimes.

### Evidencia y límites

- **UPSTREAM VERIFIED:** Hook default ZZMI y Direct Inject en Custom Launch según el flujo fijado de XXMI Launcher v2.2.1. El precedente Direct Inject de WWMI no se extrapola a ZZMI.
- **UPSTREAM DOCUMENTED:** en el código Wine consultado el Hook global vive en el scope del desktop administrado por wineserver; distintos `WINEPREFIX` permiten sesiones independientes. El comportamiento específico de Proton/`3dmloader.dll` no se comprobó.
- **HOST TEST previo 0.7:** Direct Inject y DLL remota pasaron únicamente contra el target fixture de LXMI. No se repitió en 0.8.1.
- **NO COMPROBADO / BLOCKED:** Hook bajo Proton, Direct Inject con ZZMI/ZZZ, visibilidad de procesos del contexto Steam Linux, same-prefix/same-wineserver y compatibilidad Linux/Proton del juego.
- En 0.8.1 no se ejecutó Hook, Direct Inject, Steam, Proton, Wine o ZZZ y no se modificaron Steam, juego, compatdata ni prefix.
- La verificación nativa y los comandos exactos están en `SDLC/06_Verificacion/verificacion_0_8.md`.

## LXMI-0.8.2 — Steam / Proton Process Context Evidence (2026-09-30)

### Implementado

- `lxmi-steam` reutiliza Valve KeyValues para inspeccionar en modo read-only `config/config.vdf`, con límite de 2 MiB y clasificación de config ausente, inválido, ilegible, inseguro o demasiado grande.
- Mapping por AppID, mapping global `0` y compatibility tools descubiertas permanecen separados. Coincidencia de ID instalado es evidencia de configuración/disponibilidad, no selección efectiva.
- Se añadió `SteamProtonProcessContextEvidence` con fuentes/findings de manifest, biblioteca, config, compatdata y pfx; runtime/prefix/contexto efectivos y relaciones loader/prefix/wineserver permanecen `Unknown`.
- La readiness agrega requisitos y componentes del gate independientes; `NeedsEvidence`, `NoDecision`, gate no elegible y guard Hook se conservan.
- Advanced → ZZZ Readiness expone el mapping y fuentes bajo detalles técnicos. Sin UI manual nativa en esta verificación.
- Se fijó la app en `0.8.2`; no se implementó lanzamiento, ni se ejecutó proceso externo, ni se modificó Steam, ZZZ, compatdata, prefix o runtime administrado.

### Evidencia y validación

- **LOCAL VERIFIED:** el bloque `CompatToolMapping` leído en `/home/university/.local/share/Steam/config/config.vdf` está vacío; los `localconfig.vdf` consultados no contienen esa clave. Por tanto, no se encontró override explícito por AppID en las fuentes comprobadas; el runtime efectivo continúa `Unknown`.
- **LOCAL VERIFIED:** manifest AppID `4162040`, ejecutable de ZZZ, directorios `compatdata/4162040/pfx` y Proton Experimental con metadata estructural existen en el host. Proton instalado no se promueve a selección.
- **FIXTURE:** parser/readiness cubiertos por mapping explícito/global, mapping global sin override, config malformed/missing, varios candidates y compatdata/pfx presentes/ausentes.
- Pasaron los quality gates Rust y frontend; total workspace `194 passed`, `7 ignored`. Resultados exactos y límites en `SDLC/06_Verificacion/verificacion_0_8_2.md`.

## LXMI-0.8.3 — Passive ZZZ Runtime Observation (CERRADO, 2026-09-30)

### Implementación en el árbol de trabajo

- Se añadió un observer procfs efímero, con consentimiento explícito, límite de 90 segundos, muestreo de 1 segundo y filtro de UID actual. El identificador combina nombre ejecutable esperado con evidencia independiente; múltiples matches permanecen ambiguos.
- Se leen solamente `STEAM_COMPAT_APP_ID`, `SteamAppId`, `SteamGameId`, `STEAM_COMPAT_DATA_PATH` y `WINEPREFIX` de procesos relevantes. No se lee memoria ni se controla el proceso observado.
- Se conectó el summary a ZZZ readiness sin promover Proton instalado/configurado a selección ni modificar `GameRuntimePlan.selected`. Proton observado por proceso, compatdata, prefix derivado/directo y wineserver se conservan con evidencia separada.
- Advanced incorpora consentimiento, Start/Stop y exposición de evidencia acotada. El usuario debe iniciar ZZZ manualmente desde Steam, sin mods.

### Estado de verificación

- Rust format/check/clippy/tests pasan (221 passed, 8 ignored); frontend typecheck/lint/format/build y `git diff --check` pasan. `tauri:dev` compiló y arrancó 0.8.3. Los resultados se registran en `SDLC/06_Verificacion/verificacion_0_8_3.md`.
- La observación manual completada desde la ventana Tauri se registra abajo; los metadatos de la aplicación se actualizaron a `0.8.3` solo después de esa evidencia.

### Diagnóstico de resultado silencioso (2026-09-30)

- La sesión manual reportada por el usuario se inició con ZZZ ya abierto. Los logs de la versión anterior registraron el inicio y la finalización aproximada a los 91 segundos, sin escaneos, resultado terminal o evidencia de snapshot. **UNKNOWN:** si detectó ZZZ, expiró sin match, encontró ambigüedad o no pudo leer el entorno.
- **COMPROBADO por código:** el scan inicial ya ocurría antes de dormir; el command Tauri devuelve el snapshot y React lo retenía mientras el panel estaba montado. El único lugar de render era el cuerpo del `<details>` Advanced. La extracción que solo mostró summaries no demuestra pérdida del DTO.
- Se agregó un resumen persistente visible fuera del disclosure, estado `active`/elapsed, resultado explícito (`Completed`, `ProcessNotFound`, `TimedOut`, `AmbiguousProcess`, etc.), detalle de error con código y contadores agregados de scan. Tauri registra el primer resultado y transiciones sin volcar `cmdline` ni variables.
- Se distingue permiso denegado al leer `environ`, metadatos inaccesibles, nombre no coincidente y evidencia independiente insuficiente. El candidato busca nombre esperado en las señales de proceso disponibles, incluso cuando `/proc/<pid>/exe` sea Wine/preloader.
- **FIXTURE:** un ZZZ preexistente en la primera lectura se identifica con `comm=wine64-preloader`, `cmdline` con el EXE esperado y AppID/compatdata en ascendencia Proton. También se verifica la serialización del snapshot y que el stop no borra evidencia.
- **LOCAL OBSERVED — Tauri nativo (evidencia del usuario):** proceso `ZenlessZoneZero.exe`, PID `691954`; AppID `4162040`; Proton Experimental ID `1790244902`, build `experimental-11.0-20260924-x86_64`; compatdata coincidente; prefix observado; ascendencia Steam observada.
- El primer scan fue `WaitingForGame` (191 procesos, cero candidatos); el scan de `00:52:42Z` observó cuatro candidatos y aceptó uno. A los 67 segundos la UI mostró el snapshot `Observed · En curso`. Se demuestra detección durante la ventana; que ZZZ ya estuviera presente en el primer scan no se afirma.
- **LOCAL OBSERVED:** un wineserver del UID actual mostró el mismo `WINEPREFIX`, con la advertencia visible de que no prueba que ZZZ utilice esa instancia. El mapping por AppID no apareció en la fuente inspeccionada; el default global se mantuvo separado.
- **INFERIDO:** `PermissionDenied` y cero denegaciones entre candidatos directos son compatibles porque el estado del contexto agrega ascendencia, mientras ese contador no. La salida no identifica el PID denegado. Se añadió distinción de estos alcances en los diagnósticos/UI.
- **UNKNOWN:** estado terminal de la sesión después de la captura activa a 67 s, relación efectiva de ZZZ con una instancia concreta de wineserver, compatibilidad ZZMI/ZZZ bajo Steam Linux/Proton y requisitos same-prefix/same-wineserver del loader.
- `GameRuntimePlan.selected` permanece `Unknown`; loader strategy `NoDecision`; readiness `NeedsEvidence`/gate cerrado. La observación no lanzó ni controló ZZZ ni ejecutó Hook, Direct Inject, Proton de laboratorio, helper o loader.
- El observer no tiene camino de escritura a Steam/juego/prefix. No se tomó hash pre/post de esos árboles; no se reclama una comparación de filesystem.
- 0.8.3 se cerró con versión `0.8.3`; resultados y límites completos en `SDLC/06_Verificacion/verificacion_0_8_3.md`.

## LXMI-0.8.4 — Proton/Wine Context Join (2026-09-30 local)

- Peers propios MinGW + named event Local con nonce; sin process enumeration/open, memoria, loaders o red.
- Context Lab reutiliza bridge, dosdevices y Proton discovery; prefixes por sesión, argv directo, hash inventory y protección de paths. Captura no bloqueante y timeout propio del launcher sin señales wineserver/grupos en este lab.
- Modelo de contexto y comparación independiente; mappings completos separados del marker mapping; requisitos futuros NotEstablished y GameContextJoinPlan declarativo.
- HOST TEST: mismo prefix compartió evento; control de prefixes distintos no lo encontró; marker legible en ambos. Identidad wineserver permanece Unknown.
- Tauri nativo recorrió ambos casos vía accesibilidad; resultados persistieron en React. Readiness se evaluó read-only sin abrir el gate. App version 0.8.4.
- Calidad: 236 tests offline, 9 ignored, Rust/frontend gates. Referencia: [verificación](../06_Verificacion/verificacion_0_8_4.md). No se inició 0.9, juego o loaders.

## LXMI 0.8.5 — Direct Inject Context Boundary (PARCIAL / BLOCKED)

- Fixture same-prefix: official `3dmloader.dll` Inject carga la DLL LXMI; marker, nonce y paths verificados.
- Control different-prefix: filesystem visible, pipe inaccesible (Win32 2), PID numérico homónimo observado. Guard bloquea upstream antes de seleccionar un proceso no correlacionado.
- Nuevo runner cerrado, handshake pipe/PID, handle query-only del target ya verificado, artefactos por sesión y panel en Context Lab.
- Modelos de resultados/capabilities/requirements y GameContextJoinPlan declarativo; no selección loader ni autorización game.
- 11 tests offline específicos; Tauri nativo y gates registrados en verificacion_0_8_5.md.
- Cross-prefix API no invocada por falta de binding seguro: incremento abierto, versión conserva 0.8.4. No trabajo 0.9.
