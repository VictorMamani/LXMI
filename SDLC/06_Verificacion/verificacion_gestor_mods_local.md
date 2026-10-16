# Verificación — gestor local de mods y perfiles

Continuación vigente: [runtimes, perfiles reversibles, mods reales ZZZ y diseño del prototipo](verificacion_activacion_runtime.md). Los apartados siguientes conservan la evidencia histórica del gestor inicial.

Fecha local: 2026-10-14. Producto 0.8.7, sin cambio de versión. Este incremento no ejecutó Proton/Wine, helpers, loaders o juegos ni modificó sus instalaciones o prefixes. Se preservaron los cambios de trabajo previos y el historial de 0.8.8; no se hizo commit ni push.

## Backend y compilación

Desde `05_Construccion/aplicacion` pasaron:

- `cargo fmt --all -- --check`.
- `cargo check --workspace --offline`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`.
- `cargo test --workspace --offline`: **390 passed, 0 failed, 12 ignored**. No se ejecutaron las pruebas host opt-in.

Las 16 pruebas nuevas de `crates/lxmi-xxmi/tests/mod_library.rs` usan exclusivamente archivos sintéticos bajo directorios temporales. Cubren listado sin escritura, asignación por juego, importación/deduplicación folder/ZIP, preservación byte a byte, permisos privados, traversal/limpieza, symlinks/hardlinks/case collisions, nombres Windows, límites, EXE/DLL rechazados, source solapado, persistencia y selección reversible, snapshots inmutables, perfil vacío, referencias cruzadas/desconocidas/duplicadas, revisiones atrasadas, ocho escritores concurrentes, payload alterado, copia preparada alterada y symlink de metadata.

El adaptador Tauri compiló con el workspace. Los roots se derivan en backend por `store_for`, que mantiene las fronteras existentes con Steam/juegos/prefixes; no se añadió un comando con destino arbitrario.

## Frontend

Desde `apps/lxmi-desktop` pasaron `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`. `git diff --check` pasó. El detector mecánico de Impeccable devolvió `[]` para los archivos de interfaz modificados.

Playwright recorrió la interfaz Vite con **IPC Tauri simulado en memoria**. No se invocó el backend Rust desde ese navegador. Se comprobó:

1. Biblioteca vacía y carga de mods sintéticos.
2. Importación, error por ausencia de INI y deduplicación visible.
3. Marcado/desmarcado, guardado/actualización y recuperación de perfiles.
4. Búsqueda sin coincidencias y retorno a la lista.
5. Cambio a ZZZ con biblioteca vacía independiente y retorno a WuWa con perfil guardado.
6. Preparación bloqueada con selección sin guardar y resultado/ruta al preparar una selección guardada.
7. Recuperación directa del perfil actualmente editado y formato `1 archivo · 1 KiB`.
8. Tabulación del nombre de mod al campo de ruta; sin desbordamiento horizontal en 1280×1000 y 390×844.

Capturas locales con datos sintéticos: `.impeccable/review/desktop.png`, `mobile.png`, `manager-desktop.png` y `manager-mobile.png`, desde la raíz del proyecto. La revisión visual se limitó al panel nuevo, heredando el diseño existente. Las capturas no acreditan compatibilidad con el juego.

Revisión independiente de Impeccable: `Ship` en la extensión; el pase acotado posterior confirmó resueltas sus tres observaciones menores (recuperación directa, singular/plural y tamaños de archivos pequeños). No se interpreta como verificación de IPC nativo.

## Límites de la evidencia

La comprobación inicial descrita arriba usa IPC simulado. La continuación nativa siguiente cierra ese pendiente, sin acreditar compatibilidad de juego. No se probó un mod real ni instalación/activación en runtime, recarga o conflictos semánticos. Preparar genera un snapshot privado; no cambia el perfil activo del juego. No se ejecutó un experimento adicional de prefix ni se consumió consentimiento de 0.8.8.

## Continuación — IPC nativo y persistencia (2026-10-14)

Se ejecutó `mod_manager_native_smoke` con GTK/WebKit real y el mismo builder y registro de comandos que utiliza la aplicación. La automatización emite eventos DOM dentro de su propio webview; no usa mocks de `invoke` ni eventos de teclado del escritorio. Un intento previo de automatización de teclado se abandonó tras interferencia de foco; no forma parte de los resultados PASS y sus scripts no se incorporaron al proyecto.

La primera ejecución cerró con código 0 y verificó importación de carpetas, deduplicación, perfil de dos mods, bloqueo de preparación con cambios sin guardar, recuperación de la selección, snapshot real, error por ausencia de INI, importación/perfil independiente ZZZ y retorno a WuWa. La segunda ejecución, con otra instancia de la aplicación y el mismo XDG privado, cerró con código 0 y verificó persistencia de ambas bibliotecas/perfiles, reutilización del snapshot e importación/extracción ZIP por la interfaz nativa.

La inspección independiente de disco confirmó todos los hashes de payloads y snapshot, conservación byte a byte de los originales y un único snapshot tras repetir la preparación. Resultado: 3 mods sintéticos en WuWa, 1 en ZZZ y perfiles separados. Se usaron data/cache privados bajo `/tmp`; no se importó nada a la biblioteca habitual.

Evidencia durable: [gestor_mods_nativo.json](evidencias/gestor_mods_nativo.json). Reproducción: `bash tools/test-mod-manager-native.sh` con Vite abierto, desde la aplicación. La prueba usa un ejemplo de desarrollo opt-in y permanece fuera de los tests normales.

Tras extraer el builder compartido se repitieron fmt, check workspace con all-targets, Clippy workspace con all-targets y `-D warnings`, tests workspace (**390 passed, 0 failed, 12 ignored**) y typecheck/lint/format/build frontend. También pasaron `node --check` del script del ejemplo, `bash -n` del runner y `git diff --check`. No se añadió una dependencia.

Uso y pendientes: [gestor local](../05_Construccion/gestor_mods_local.md).
