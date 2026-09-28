# Verificación LXMI 0.5

Fecha de inicio de verificación: 2026-09-27; última comprobación local: 2026-09-28 (America/La_Paz). No se ejecutó contenido de paquetes ni juegos.

## COMPROBADO — quality gates

Desde `SDLC/05_Construccion/aplicacion`:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Resultado: pasan. 114 tests Rust: 69 previos + 45 nuevos. Clippy encontró inicialmente un clone innecesario en un test; se corrigió con `std::slice::from_ref`, sin desactivar reglas.

Desde `apps/lxmi-desktop`:

```bash
npm run typecheck
npm run lint
npm run format:check
npm run build
npm run tauri:dev
```

Typecheck, lint, Prettier y Vite build pasan. Tauri compiló e inició ventana nativa v0.5.0. La captura nativa confirmó el arranque. La sección nueva se comprobó adicionalmente en el navegador para controles, labels y ausencia de overflow horizontal; esa vista no dispone del bridge nativo y **no prueba IPC**.

**NO COMPROBADO:** recorrido visual completo de importación y revisión mediante clics en la ventana nativa. El backend real de import/plan sí se ejercitó con el ejemplo Rust siguiente. No declarar ese recorrido como prueba E2E de UI.

## FIXTURE — cobertura nueva

`crates/lxmi-xxmi/tests/foundation.rs` crea árboles temporales y payloads de texto originales, sin Steam ni DLLs reales. Cubre:

- WWMI válido, missing/empty files, namespaces desconocidos y juego incorrecto.
- Libraries separadas, manifiesto inválido y DLL aislada insuficiente.
- Hash SHA-256 conocido, cambio de archivo, manifest alterado, verificación antes de planificar y duplicados sin overwrite.
- Traversal, absolutos, separadores Windows, nombres ambiguos, symlinks de archivo/ancestro/storage/destino, hardlinks y permisos.
- Límites por archivo/total/entradas/profundidad, archives rechazados y almacenamiento XDG.
- Staging/promoción, payload separado de metadata, fuente preservada e import inválido no publicado.
- Discovery ausente/presente/incompleto/ilegible y snapshots antes/después que demuestran ausencia de escrituras.
- Plan revisable, replacements con backups, juego ausente y candidatos que no prueban compatibilidad.
- Un archivo ejecutable sintético nunca se ejecuta durante import/validación.

No se ensayaron corte de energía ni ataques concurrentes de un proceso hostil con los mismos privilegios.

## UPSTREAM VERIFIED — corpus real externo

Se descargaron a `/tmp/lxmi-upstream` WWMI v1.0.0 y XXMI Libraries v1.1.7; se inspeccionó su inventario sin ejecutar binarios. La extracción de investigación se hizo en `/tmp` con comprobación de rutas y tamaños. **LXMI no tiene extractor ZIP**.

Con los directorios extraídos y Manifest.json oficial al lado de las DLL:

```bash
cargo run -p lxmi-xxmi --example local_review -- /tmp/lxmi-validation/lxmi /tmp/lxmi-upstream/verified-wwmi /tmp/lxmi-upstream/verified-libs
```

Resultado: validación estructural e importación completadas, paquetes revalidados por hash, plan generado. Los artefactos, inventarios y evidencia completa de esa sesión viven en `/tmp`, no en Git. El ejemplo requiere que exista el padre del store y es una herramienta de desarrollo con import explícito, no un instalador del juego.

## COMPROBADO — Wuthering Waves real

Observación de integración del 2026-09-27, conservada como histórica:

- Steam encontró AppID 3513350 y directorio de juego presente.
- `Client/Binaries/Win64` no existía en esta observación; no se confirmó completitud/utilidad de instalación.
- `compatdata/3513350` y `pfx` ausentes; estado normal `NotInitialized`.
- Selección Proton: Unknown; readiness de plataforma NeedsInitialization.
- Sin WWMI/Libraries detectados en raíz del juego, subcarpeta WWMI y ubicación Win64 esperada.
- El plan resolvió el ejecutable como **ruta candidata**, no existente verificado, y destinó archivos a `/tmp/lxmi-validation/lxmi/runtimes/wwmi`. `executable=false`; no creó ese destino.
- Ninguna escritura del import o plan se dirigió a Steam/juego/prefix. No se lanzó Steam, Proton, Wine ni Wuthering Waves.

La ausencia en esos candidatos no descarta un XXMI Launcher externo. Firmas y compatibilidad WWMI/Proton: **NO COMPROBADO**.

## COMPROBADO — scan actual del host (2026-09-28)

- `npm run tauri:dev` compiló/inició el binario nativo LXMI 0.5.0; Vite respondió en `http://127.0.0.1:1420/`. El proceso se detuvo al terminar el smoke test.
- En el scan actual, la biblioteca enumerada presentó manifests `1493710`, `4162040` y `4183110`; no había manifest `3513350`, por lo que Wuthering Waves no fue detectado como instalación actual.
- Existen carpetas `Proton - Experimental` y `Proton Hotfix`, pero la inspección limitada encontró solo archivos auxiliares `dist.lock`, `files/steampipe_fixups_mtime` y `__pycache__`; no encontró `toolmanifest.vdf`, `compatibilitytool.vdf` ni entrypoint `proton`. No se clasificaron como runtimes válidos.
- `compatdata/3513350` y `pfx` no existen. El planner de la entrada del catálogo devolvió selección `Unknown`, prefix `Unknown` y readiness `Blocked` al no encontrar instalación de juego.
- Las herramientas Steam Linux Runtime no se clasificaron como Proton ni se infirió un candidato. No se escribió en Steam, juego ni prefix.

Este resultado actual reemplaza cualquier inferencia de que el juego o Proton siguen instalados en el host; la observación del 2026-09-27 se conserva como histórica. La prueba no valida un entorno de juego listo.
