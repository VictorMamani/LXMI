# LXMI 0.6 — Verificación del Runtime Bridge

Fecha: 2026-09-28

## Propósito

Probar por separado el protocolo y las restricciones del bridge Windows. Esta verificación no evalúa compatibilidad de ZZZ, XXMI, inyección ni juego.

## Revisión de repositorio

- Root Git: /home/university/BYTE-CX/04_Proyectos/19_LXMI
- Branch: main
- Remote: origin = git@github.com:VictorMamani/LXMI.git
- Checkpoint de partida: 844008b (feat: add managed ZZMI runtime topology)
- Estado al cerrar 0.6: cambios del incremento presentes y sin commit; se conservaron. No se publicó nada.

## Tests por fixtures

Los tests Rust no necesitan Steam, Proton, Wine ni red. Incluyen:

- request/response JSON, versión del protocolo, nonce y mismatch;
- helper solo lee marker JSON y rechaza ruta relativa insegura, ruta Windows/absoluta, symlink y marker inválido;
- helper staged/hash local, ruta de runtime administrado, argv de Proton y allowlist de environment;
- rechazo del contexto compatdata de juego;
- prefix aislado sin layout Wine fabricado por LXMI;
- timeout tipado, terminación del proceso hijo/grupo y output cap drenado;
- ejecución mediante executor falso y comprobación de que una selección bridge explícita no altera runtime selection del juego.

**FIXTURE:** las respuestas, hashes y paths de estos casos son sintéticos. No equivalen a una ejecución real del helper bajo Wine/Proton.

## Cross-build

**COMPROBADO:** toolchain activo `rustc 1.98.1 (48a229cea 2026-09-01)` y `cargo 1.98.1 (797e8a9bc 2026-08-05)`. Están instalados los targets `x86_64-unknown-linux-gnu` y `x86_64-pc-windows-gnu`. El host dispone de `/usr/bin/x86_64-w64-mingw32-gcc`, GCC `13-win32`.

El intento histórico con Zig 0.15.2 sí falló al enlazar porque no encontró `msvcrt`. Se resolvió usando MinGW como linker explícito; no se cambió el linker Linux ni se instaló paquetes con sudo. La compilación reproducible es:

~~~bash
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build --manifest-path tools/lxmi-bridge-helper/Cargo.toml \
  --target x86_64-pc-windows-gnu --release
~~~

**COMPROBADO:** produjo `target/x86_64-pc-windows-gnu/release/lxmi-bridge-helper.exe`, PE32+ Windows x86-64, 1,517,755 bytes. SHA-256:

~~~bash
2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2
~~~

El helper se staged con `bash tools/install-bridge-helper.sh` bajo `$XDG_DATA_HOME/lxmi/helpers/`; el SHA-256 del staged coincide con el artefacto. El sidecar de LXMI registra target, versión de helper, protocolo y hash. El `.exe` no se ejecutó nativamente en Linux.

Dependencias de host para otros equipos: `gcc-mingw-w64-x86-64` y `mingw-w64-x86-64-dev`. En este host MinGW ya estaba disponible.

## Host bridge test

**HOST TEST — COMPROBADO:** se lanzó el helper inocuo por la Proton seleccionada explícitamente para pruebas, `Proton Experimental`, versión `experimental-11.0-20260924-x86_64`. La invocación usa `proton runinprefix <helper.exe>`; no usa `proton run`, que inicia `steam.exe` y en el primer ensayo expiró por timeout. El juego conserva `GameRuntimePlan.selected = Unknown`.

La corrida positiva se repitió después de reconstruir/stagear el helper. Resultado estructurado:

- helper v`0.6.0`, protocolo `1`, PE/hash esperado e informado iguales;
- handshake `succeeded`, nonce aleatorio nuevo devuelto sin cambios;
- proceso exit `0`, duración observada `441 ms`;
- path visibility `mapped_and_readable`, marker JSON legible y environment marker coincidente.

La ruta Linux `/home/university/.local/share/lxmi/runtimes/zenless-zone-zero/zzmi/zzmi-6dc676fdfc0d30f7e69de43b9123936e7750b0e58baadb384018815a2891f052` fue visible desde el helper como `Z:\home\university\.local\share\lxmi\runtimes\zenless-zone-zero\zzmi\zzmi-6dc676fdfc0d30f7e69de43b9123936e7750b0e58baadb384018815a2891f052`. No se dio por válido el mapping solo por existir `Z:`: el propio helper abrió el manifiesto.

SHA-256 del `runtime-manifest.json`, leído por Linux y por el helper Windows, coincide:

~~~text
c86d41865cc81c7639eecc17f021d80448fd2e12900453cc66f37d00c5332a5e
~~~

**HOST TEST negativo — COMPROBADO:** con una ruta relativa controlada pero inexistente dentro del storage LXMI, el helper devolvió JSON con protocolo `1`, nonce correlacionado, `success=false`, `runtime_visible=false`, `runtime_manifest_readable=false` y diagnóstico `runtime_not_visible`; salió con código `2`. No hubo panic ni timeout.

El test host de Rust usa `PrefixMode::IsolatedTemporaryPrefix` en `$XDG_DATA_HOME/lxmi/test-prefixes/bridge-v1/compatdata`. El helper solo puede acceder al managed storage y lee el marker; no enumera procesos ni abre otro proceso.

## Mutación de ZZZ / Steam

**COMPROBADO:** ni ZZZ ni `ZenlessZoneZero.exe` se pasaron al helper o a Proton como ejecutable; no se lanzó ni inspeccionó como proceso de juego. El helper no implementa apertura de procesos, carga de DLL o inyección. Antes/después de la prueba quedaron iguales la metadata del ejecutable y los targets vigilados (`d3d11.dll`, `d3dcompiler_47.dll`, `d3dx.ini`, `ZZMI`); los cuatro siguen ausentes. `compatdata/4162040/pfx` no se usó y su snapshot quedó sin cambios.

Proton sí inicializó/actualizó el prefix aislado LXMI y ejecutó su mantenimiento propio. Su distribución Proton tocó el timestamp de `dist.lock`; no se alteró la configuración de Steam. Este efecto está dentro de lo advertido y aceptado en la UI.

## UI

**COMPROBADO en ventana Tauri nativa (no Vite/browser-only):** `npm run tauri:dev` abrió LXMI. Se ejecutaron `Scan Steam`, se confirmó ZZZ/AppID `4162040` y el ejecutable esperado, se inspeccionaron los packages administrados, se eligió explícitamente Proton Experimental, se aceptó la advertencia de efectos y se lanzó Runtime Bridge Test. La ventana mostró helper disponible, protocolo v1, handshake pasado, path legible y SHA-256 del manifiesto coincidente. La selección Proton del juego permaneció `unknown`.

También se verificaron manualmente los estados de diagnóstico:

- helper temporalmente ausente del managed storage: UI mostró `Helper: missing` e instrucciones de preparación; se restauró de inmediato y el hash volvió a coincidir;
- paquetes sin seleccionar: UI mostró `Runtime ZZMI administrado: no disponible para los paquetes seleccionados`, Proton explícito sin elegir y selección del juego `unknown`; no se ejecutó Proton en este estado.
- sin Proton disponible: se inició una segunda ventana Tauri con `HOME` y XDG temporales vacíos; tras inspeccionar el bridge, la UI mostró `No se encontró un Proton válido con entrypoint ejecutable` y no ofreció candidato. El entorno real de Steam no se consultó ni alteró durante este caso.
- El primer recorrido del estado helper ausente reveló un `<p>` anidado inválido. Se corrigió el JSX; el estado se repitió en Tauri y no volvió a registrar warning React/Vite. Typecheck, lint, Prettier y build se repitieron después del fix.

El prototipo visual separado no se modificó.

## Resultados de quality gates

| Comando | Resultado |
|---|---|
| `cargo fmt --all -- --check` | PASÓ |
| `cargo check --workspace` | PASÓ |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASÓ |
| `cargo test --workspace` | PASÓ: 166 tests pasaron; 6 tests opt-in/host/network ignorados por defecto |
| `cargo build --manifest-path tools/lxmi-bridge-helper/Cargo.toml --target x86_64-pc-windows-gnu --release` con linker MinGW | PASÓ: PE32+ generado y hash confirmado |
| `npm run typecheck` | PASÓ |
| `npm run lint` | PASÓ |
| `npm run format:check` | PASÓ |
| `npm run build` | PASÓ: Vite produjo build de producción |
| `npm run tauri:dev` | PASÓ: ventana nativa abierta e interacción manual bridge realizada |
| `cargo test -p lxmi-bridge --test proton_host -- --ignored --nocapture` | PASÓ: handshake host por Proton; ejecución opt-in aislada |
| Prueba host negativa directa | PASÓ: JSON `runtime_not_visible`, nonce correlacionado, exit 2 |
| `git diff --check` | PASÓ después de los cambios de código y documentación |

## Clasificación

- **FIXTURE:** tests unitarios con executor falso, marker y respuestas sintéticas.
- **COMPROBADO:** Rust workspace, frontend, build PE, staging/hash y ventana Tauri nativa.
- **HOST TEST:** Proton Experimental + helper Windows + mapping `Z:` validado por lectura/hash real; prueba negativa estructurada.
- **UPSTREAM VERIFIED:** documentación del script de Proton y semántica de `runinprefix` en `01_Descubrimiento/bridge_linux_windows.md`.
- **NO COMPROBADO:** compatibilidad de ZZMI/ZZZ/Steam/Linux, carga de DLL, loader, interacción con proceso del juego y soporte del runtime para jugar con mods.

## Cierre técnico

LXMI 0.6 queda cerrado: el helper Windows se enlazó con MinGW, fue staged e identificado por hash; el handshake host positivo y negativo pasaron por una Proton elegida explícitamente y prefix aislado; la ventana Tauri mostró esos resultados. La selección Proton del juego sigue `Unknown`. No se inició LXMI 0.7.
