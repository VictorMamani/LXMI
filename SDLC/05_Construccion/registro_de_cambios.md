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
