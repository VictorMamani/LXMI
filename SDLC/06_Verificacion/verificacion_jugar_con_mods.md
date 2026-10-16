# Verificación de «Jugar con mods» (ZZZ)

Fecha local: 2026-10-15. Versión de la aplicación: 0.8.7 (sin cambiar).

## Resultado en el juego real

Sesión `zzz-direct-f8acde73c92b4a6559573efd2831c484`, con el runner del
experimento en modo `--hook` y el perfil **ZZZ — Miyabi y RabbitFX**:
`HookLibrary` 0, ZZMI presente por ruta exacta en el ZZZ nuevo, `UnhookLibrary`
0 y proceso original vivo tras 15 s. El contexto Steam del juego se volvió a
verificar. El log de ZZMI no tiene errores y el log Proton no contiene
`c0000005`, `c0000409` ni `c0000135`. **El usuario entró en partida y vio los
mods de Miyabi aplicados.** [Resumen](evidencias/zzz_hook_mods_visibles_20261015.json).

Corrección registrada: los avisos de DXVK «Unknown interface query» aparecen
también en esta ejecución correcta (289); no discriminan el fallo con Inject.

## Integración en la aplicación

| Pieza | Comprobación |
|---|---|
| `crates/lxmi-launch` | 16 tests: reescritura de `[Loader]`, protocolo del helper, contexto procfs (letra de unidad Steam, AppID, prefix, duplicados, Proton/SLR) y máquina de estados (éxito, abrir Steam, juego abierto, salida con código, fallos, cancelación) |
| `copy_profile_runtime_for_launch` | Copia real del perfil seleccionado: 90 archivos reinventariados, `loader = LXMI Loader.exe`, resto de `d3dx.ini` idéntico, único `.exe` el helper, permisos 700 |
| Helper `tools/lxmi-mod-loader/loader.c` | `-Wall -Wextra -Werror`; incrustado vía `build.rs`. En un prefix de prueba LXMI: `HookLibrary` 0 y armado; tras **Cancelar** (SIGTERM al grupo propio) no queda ningún proceso del cargador |
| UI nativa GTK/WebKit + IPC real | `selected_profile_native_review`: perfil restaurado, `launch_with_mods_available = true`, sin bloqueos, botón «Jugar con mods» habilitado, sin sesión activa. No se pulsó el botón. [Informe](evidencias/jugar_con_mods_nativo_20261015.json) |
| Workspace | `cargo fmt --check`, Clippy `-D warnings`, `cargo test --workspace` 409 PASS / 12 ignored; frontend typecheck, lint, format y build PASS |

## Primer intento desde el botón: corrección AppArmor

En los dos primeros intentos desde la app, el juego arrancó sin mods: el
cargador no llegó a ejecutarse. `helper.stderr.log` contenía `bwrap: setting up
uid map: Permission denied`. Ubuntu tiene `kernel.apparmor_restrict_unprivileged_userns
= 1`; la app corría `unconfined` y no podía crear el contenedor del SLR, mientras
que los experimentos anteriores se lanzaron desde un shell con perfil
`nautilus`. Reproducido con `aa-exec -p unconfined`.

Corrección: el cargador se pide al servicio de Steam
(`steam-runtime-launch-client --alongside-steam`, perfil `steam`). Ese servicio
solo reenvía la señal al hijo directo, así que el comando anuncia su grupo y
**Cancelar** lo termina entero. Si el cargador termina sin eventos, la app
muestra la causa de su stderr.

Prueba `fixture_helper_arms_and_cancels_through_steam` sin perfil AppArmor, en
el prefix de fixture (no el del juego): grupo anunciado, `HookLibrary` 0,
armado y, tras cancelar, ningún proceso del cargador en 2,1 s. Workspace: fmt,
Clippy `-D warnings`, 413 PASS / 13 ignored.
[Evidencia](evidencias/jugar_con_mods_apparmor_20261015.json).

## Primer inicio completo desde el botón

Con la corrección AppArmor, el botón cargó el perfil de 12 mods (543 archivos):
grupo anunciado, `HookLibrary` 0, ZZMI en el juego a los 2,8 s, `UnhookLibrary`
0 y juego vivo tras 15 s. **El usuario vio todos los mods cargados.** Steam ya
había actualizado Proton Experimental a `experimental-11.0-20261014`.

Después, al abrir el menú, el juego se volvió lento y se cerró. El kernel lo
terminó por falta de memoria (`Out of memory: Killed process … ZenlessZoneZero
… anon-rss:5592788kB`); el log de Proton no muestra una excepción del juego. El
equipo tiene 15 GiB de RAM y GPU integrada Iris Xe; los mods ocupan 2,5 GiB (uno
solo, 1,2 GiB). Que el tamaño de los mods contribuyera es una inferencia, no
está medido.

Las letras rojas son avisos de 3DMigoto (`show_warnings = 1`). El texto exacto
no se conserva; el análisis estático de los `.ini` encontró 18 recursos que
apuntan a archivos ausentes en cuatro mods y dos `TextureOverride` repetidas en
Remielle, sin conflictos de hash entre mods (las `[Constants]`/`[Present]`
repetidas son válidas en mods). `tools/mod-repair` genera copias reparadas:
Remielle pasa de 1,2 GB a 454 MB y ninguna copia tiene recursos ausentes; su
carga en el juego está pendiente. Los archivos tampoco están en la biblioteca: no
venían en los paquetes. [Evidencia](evidencias/jugar_con_mods_boton_12_mods_20261015.json).

## Cierre en el módulo de protección

Un inicio posterior con 12 mods (cuatro reparados, 1,5 GiB) cargó ZZMI y se
cerró antes del menú con `0xc0000005`. Sin OOM. El minidump del juego sitúa la
excepción en `MHYPBase.dll+0x182a36a` (escritura en `0x1000`), el módulo de
protección del juego, sin código de ZZMI ni de los mods en la pila. La causa no
está determinada; LXMI no intenta evitar ni analizar esa protección.
[Evidencia](evidencias/zzz_cierre_mhypbase_20261015.json).

## Inicio confirmado con la colección reparada

Después de liberar espacio en el disco principal, el usuario confirmó que el
juego carga y funciona con **ZZZ — 12 mods (4 reparados)**. El arranque
`001792116936-345e02d7b0c7725c` contiene 550 archivos y 1 571 355 247 bytes;
registró `HookLibrary` 0, ZZMI presente a los 2580 ms, `UnhookLibrary` 0 y juego
vivo tras 15 segundos. LXMI informó `Running` y `mods_loaded = true`.
[Evidencia](evidencias/zzz_coleccion_reparada_en_juego_20261015.json).

## Pendiente de validación adicional

- Observar estabilidad prolongada y repetir la comparación de perfiles si
  reaparece alguno de los cierres anteriores.
- Repetir con un perfil más ligero para confirmar el efecto de la memoria.
- Avisar en LXMI, antes de jugar, de recursos ausentes y del tamaño del perfil.
- Wuthering Waves, otras versiones de Proton y estabilidad prolongada.
- Recarga sin reiniciar el juego.
