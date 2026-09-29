# LXMI 0.7 — verificación del upstream loader bajo Proton

**Fecha:** 2026-09-28  
**Alcance:** laboratorio aislado LXMI. No valida ZZMI ni carga en ZZZ.

## Etiquetas de evidencia

- **UPSTREAM VERIFIED:** inspección de fuentes oficiales fijadas y metadata de release autenticada.
- **FIXTURE / BUILD:** artefactos de prueba sintéticos propios compilados para Windows.
- **HOST TEST:** ejecución observada en el host Linux a través de Proton Experimental seleccionado explícitamente.
- **NO COMPROBADO:** afirmación fuera del alcance de este test.

## Configuración fijada

| Elemento | Valor comprobado |
|---|---|
| XXMI Launcher | v2.2.1, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a` |
| Nota histórica consultada | v2.1.5, release con commit corto `abc8087`; documenta injector custom en `3dmloader.exe` para la ruta Direct Inject indicada por esas notas |
| XXMI Libraries | v1.1.7, release `387957029`, commit `6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9` |
| Loader usado | `3dmloader.dll`, SHA-256 `44965ee51786db44fb4252671f356426ca7d879e6f7e8436b27d68e13237b0cf` |
| Runtime de prueba | Proton Experimental `experimental-11.0-20260924-x86_64`, seleccionado explícitamente desde Loader Lab |
| Prefix | `$XDG_DATA_HOME/lxmi/test-prefixes/loader-v1/compatdata/pfx` |
| Target | `$XDG_DATA_HOME/lxmi/tests/loader-v1/target/lxmi-loader-test-target.exe` |
| Test DLL | `$XDG_DATA_HOME/lxmi/tests/loader-v1/runtime/lxmi-loader-test.dll` |

**UPSTREAM VERIFIED:** en la integración fijada, ZZMI hereda el modo Hook por defecto; el Hook configura `3dmloader.dll` y usa un hook global. El test Direct Inject es una ruta distinta. El runner de prueba usó el export `Inject` del componente `3dmloader.dll` únicamente con el PID de su hijo propio. La diferencia de modo se documenta en `../01_Descubrimiento/loader_xxmi.md` y ADR-027.

**Licencia:** el package XXMI Libraries publica GPLv3; la verificación local clasifica el loader para uso ejecutable local en el test. El binario upstream no se añadió al repositorio ni se redistribuye.

## Artefactos de laboratorio

**FIXTURE / BUILD — COMPROBADO:** `bash tools/lxmi-loader-lab/build.sh` produjo ejecutables PE32+ x86-64 con MinGW GCC 13-win32. El pipeline de staging volvió a validar sus hashes:

| Artefacto | Tamaño | SHA-256 |
|---|---:|---|
| `lxmi-loader-test-runner.exe` | 268,300 bytes | `8867de4a5a98d241ea205398d529d715ebd361019a5e0152b522a1a70cfcf021` |
| `lxmi-loader-test-target.exe` | 303,541 bytes | `ea00439d03cad71316d3ba1dbe4901193d93e38a0e50d2eaedeb2c9218fb4c08` |
| `lxmi-loader-test.dll` | 220,839 bytes | `e51aae6187d6029c9ac45b0f30dae0b60cdda27f42e859170a7bff57d87071ad` |
| upstream `3dmloader.dll` | 20,480 bytes | `44965ee51786db44fb4252671f356426ca7d879e6f7e8436b27d68e13237b0cf` |

El runner solo permite los modos cerrados baseline/positivo/negativos; crea el target de nombre fijo con `CreateProcessW`, toma ese PID y pasa ese PID al export upstream. La DLL test verifica su proceso anfitrión y nonce antes de escribir su marker bajo el resultado privado del laboratorio.

## HOST TEST bajo Proton

**HOST TEST — COMPROBADO:** el host test opt-in de Rust pasó en 5.26 s bajo el Proton seleccionado. Cada modo usó `runinprefix` y el contexto aislado `loader-v1`. El runner espera que el marker pueda leerse completamente antes de validarlo, evitando aceptar una creación de archivo todavía en curso:

| Caso | Resultado observado |
|---|---|
| Baseline | target inició y reportó ready; no cargó loader/test DLL; exit `0` |
| Direct Inject positivo | target ready; export upstream retornó `0`; marker verificó proceso/nonce; exit `0` |
| Target ausente | structured failure `target_not_found`, sin fallback ni target iniciado; exit `10` |
| DLL ausente | structured failure de loader (`INVALID_DLL_PATH`, código `110`); target terminó limpiamente; exit `11` |
| Nonce incorrecto | DLL creó marker bajo nonce alterado; LXMI rechazó su validez; exit `12` |

El positivo demuestra que el loader seleccionado pudo cargar una DLL inocua ubicada fuera del directorio del proceso target bajo la build Proton probada. `dosdevices` fue inspeccionado después de iniciar Proton; el helper retornó rutas Windows del target y DLL que coincidieron con los mappings del prefix. No se confió solo en que existiera `Z:`.

Esto **no** demuestra comportamiento del Hook default de ZZMI, ejecución de `3dmloader.exe`, compatibilidad ZZMI, ZZZ, Steam ni soporte Linux/Proton del juego.

## Validación manual Tauri nativa

**HOST TEST — COMPROBADO en ventana Tauri nativa; no Vite/browser-only:**

1. Se abrió Advanced → Loader Lab y se inspeccionó package/component provenance.
2. Se seleccionó `Proton - Experimental · 1790244902 experimental-11.0-20260924-x86_64` para el laboratorio. Esto no cambió `GameRuntimePlan.selected`.
3. Se aceptó la advertencia del prefix aislado.
4. Baseline mostró target iniciado/ready, loader no, DLL no, marker no y exit `0`.
5. Direct Inject mostró target iniciado/ready, DLL cargada, marker verificado, nonce coincidente y mapping Windows.
6. El modo `Negativo · target ausente` mostró `Rechazo negativo esperado`, target no iniciado, sin marker y exit `10`.

La UI permite inspeccionar/stagear y ejecutar solo el plan de paths fijos. No presenta una acción en Home ni recibe PID/path/dll del usuario. `npm run tauri:dev` compiló e inició el binario Tauri; la prueba manual se recorrió dentro de esa ventana.

## Aislamiento y mutaciones

- **COMPROBADO por guard y host test:** el target y la DLL viven en `tests/loader-v1/`; no se usa la carpeta de juego.
- **COMPROBADO por test de regresión del servicio:** metadata del ejecutable ZZZ y compatdata/prefix `4162040` quedaron iguales antes/después del experimento host; el test no pasa ZZZ al runner ni a Proton.
- **COMPROBADO:** el prefix que Proton creó/actualizó es solo `$XDG_DATA_HOME/lxmi/test-prefixes/loader-v1/compatdata/`. La distribución de Proton puede actualizar su propio `dist.lock`.
- `3dmloader.dll`, el runner, target, test DLL, configuración y logs se quedan bajo storage privado del usuario. No se escribió en Steam, ZZZ, compatdata de juego o prefix de juego.
- No se interactuó con anti-cheat. El test no contiene enumeración de procesos, hook global, `OpenProcess` por búsqueda externa ni PID arbitrario desde UI.

## Pruebas y quality gates

Resultados finales de esta ejecución:

| Comando | Estado |
|---|---|
| `cargo fmt --all -- --check` | Pasó |
| `cargo check --workspace` | Pasó |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Pasó |
| `cargo test --workspace` | Pasó: 174 passed, 7 ignored opt-in/host/network tests |
| `cargo build --manifest-path tools/lxmi-bridge-helper/Cargo.toml --target x86_64-pc-windows-gnu --release` | Pasó con linker MinGW explícito; PE32+ x86-64, 1,517,755 bytes; SHA-256 `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2` |
| `bash tools/lxmi-loader-lab/build.sh` | Pasó; los tres PE32+ x86-64 y sus hashes están arriba |
| `npm run typecheck` | Pasó |
| `npm run lint` | Pasó |
| `npm run format:check` | Pasó |
| `npm run build` | Pasó; Vite build completado |
| `npm run tauri:dev` + flujo manual nativo | Pasó en ventana Tauri; baseline, positivo y rechazo de target ausente |
| Host test Proton opt-in | Pasó: baseline, positivo, target ausente, DLL ausente y nonce incorrecto |
| `git diff --check` | Pasó después de las actualizaciones de código y documentación |

## Estado del objetivo

- Loader Direct Inject en target de prueba con Proton: **COMPROBADO**.
- Remote DLL path bajo Proton: **COMPROBADO para el test fixture**.
- Default Hook de ZZMI, `3dmloader.exe`, ZZZ/ZZMI, Steam Linux, Proton elegido por Steam, ejecución real y comportamiento anti-cheat: **NO COMPROBADO**.
- LXMI 0.8 no está iniciado.
