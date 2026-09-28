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
