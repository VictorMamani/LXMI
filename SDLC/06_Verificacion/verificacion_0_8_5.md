# Verificación final 0.8.5.1 — cierre del producto 0.8.5

**COMPROBADO: 0.8.5 CERRADO para el experimento fixture.** Cross-prefix **BLOCKED_NOT_ATTEMPTED**, resultado concluyente y seguro admitido por 0.8.5.1. No se afirma compatibilidad ZZZ ni fallo upstream Inject. Versión producto 0.8.5.

## HOST TEST

Tool explícita Proton Experimental `1790244902 experimental-11.0-20260924-x86_64`, MinGW-w64, protocol 2 del target/runner, protocol 1 del marker DLL. Dos prefixes privados, sin Steam/game prefix. Resultados CLI detallados en [investigación](../01_Descubrimiento/direct_inject_context_boundary_0_8_5.md).

- Same-prefix nonce `30c291a7a26fc2d28ddce86250c6a91b`: target 216 / runner 232; consulta image/time/PID Verified; Inject 0; marker + nonce verified.
- Cross-prefix nonce `21d517bf8100e5af6ed23f7a6e3d0f44`: target 220 / runner 216; OpenProcess query-only falla Win32 87; PidNotVisible; API no llamada y loader no cargado. No fallback.
- Ambos leen filesystem marker con nonce. Wineserver identity UNKNOWN; no se enumera procesos para resolverla.

## Native Tauri — COMPROBADO

Ventana real `lxmi-desktop` iniciada por `npm run tauri:dev`; controles nativos por AT-SPI, no browser Vite. Scan Steam read-only, Context Lab, elegir Proton explícito, aceptar fixture y ejecutar ambos botones. Resultados retenidos simultáneamente y warning exacto visible.

- Same-prefix UI nonce `776711a951e92548113186b901834b2c`: Windows target 224 / runner 240; imagen completa coincide, FILETIME `134352995965900130` esperado y consultado; identidad Verified; Inject 0; marker PASS. Linux child launcher 836907, start ticks 8650454 (no PID Unix del target).
- Cross-prefix UI nonce `7939a1259c697a06a43b6b9139ec0c23`: target 216 / runner 220; PidNotVisible; Win32 87; no imagen/tiempo consultados; **BLOCKED_NOT_ATTEMPTED**, no Inject. Linux launcher 837639, start ticks 8651794.
- UI expone rutas exactas, creación, Windows PID, Linux launcher y motivo de no intento; filesystem legible en ambos, wineserver UNKNOWN. Readiness evaluado declarativamente: NotReady/gate not_ready, juego no lanzado, Proton selected Unknown, loader NoDecision, Hook blocked. Input de packages nulo en ese panel representa managed_runtime Missing en la evaluación, no ausencia probada en disco.

## FIXTURE / tests offline

254 passed, 0 failed, 10 ignored. Siete tests adicionales respecto al checkpoint: gate con PID/image/time correctos, wrong image/session path, creación distinta, nonce distinto, todos los estados no Verified (incluidos collision/denied/exited/not-visible), FILETIME inválido y parser de start ticks propio. Fake cross-prefix modela PID 220 simultáneo para target y runner con imagen del runner; no llama Inject. Regresiones bridge, loader/context labs y observer siguen verdes. Tests normales sin red ni /proc real para estos escenarios; host ignorados opt-in ejecutados separadamente.

## Binarios de este preflight

| Componente PE x86-64 | SHA-256 |
|---|---|
| Target | `0e8b13fe6425c9592b76143b1a8a18fec4cf3271a7c0e49275bcc422fc6901f8` |
| Context runner | `b8f33a8503d564c423a57917bfcd716d056ff77d6ffe9ddf93c13b5a141cb6da` |
| Test DLL | `295bc16b4198c8ac1afdfd3dcf779f3d655bba90d9d64171deb934bab0701d43` |
| 3dmloader.dll oficial | `44965ee51786db44fb4252671f356426ca7d879e6f7e8436b27d68e13237b0cf` |

Componente upstream de XXMI Libraries v1.1.7, release ID 387957029, commit 6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9; package/component signatures revalidadas por servicio existente antes del staging. Hash propio es integridad, no autenticidad upstream. No se redistribuyen bins oficiales en Git.

## Quality / seguridad — COMPROBADO

Rust fmt/check/Clippy -D warnings/tests workspace, frontend typecheck/lint/format:check/build, Windows fixture build MinGW y bridge helper cross-build release. Diff check y revisión final de versión se realizan al cerrar. No SYNCHRONIZE/VM permissions en preflight, process enumeration, Hook, debugger, Steam launch, acceso al PID ZZZ, game-prefix execution o custom injector. Query exclusiva del PID fixture.

Baseline antes/después: size/mtime_ns/inode de EXE ZZZ, directorio pfx4162040, system.reg, user.reg, config.vdf **sin cambios**. No hash del juego entero. Packages/managed runtime originales solo se leen y se revalidan; únicamente nuevo staging/tests privados recibe writes. Proton escribe sus prefixes aislados y puede dejar wineserver temporal; no se mata wineserver. Cierre de Tauri/Vite propio tras validación, juego no controlado.

## UNKNOWN / NO COMPROBADO

Cross-prefix operación Inject (deliberadamente no autorizada por gate fallido); endpoint wineserver exacto; requisito universal del loader; compatibilidad ZZMI/ZZZ; join del prefix real. No se inicia 0.8.6/0.9. Branch main, checkpoint 16f0c56 conservado; cambios de este incremento sin commit ni push.

## Confirmación con versión final 0.8.5

La repetición tras el bump ejecutó ambos controles en la ventana nativa y re-evaluó readiness sin ejecutar el juego:

| Caso | Nonce | Windows PID target / runner | Identidad | API |
|---|---|---|---|---|
| Same-prefix | `218c7e91f6fa0774307c6c8f7fba17b4` | 220 / 236 | Verified; FILETIME `134352998381247030` e imagen exacta coinciden | Inject 0, marker + nonce PASS |
| Cross-prefix | `4b7fd74cdecc7b630c90734d28cace7b` | 220 / 216 | PidNotVisible; Win32 87; imagen/creación consultadas no disponibles | BLOCKED_NOT_ATTEMPTED |

Tauri/Vite propios cerrados después de la revisión. Baseline protegido volvió a coincidir. Todos los quality gates pasaron sobre metadata producto 0.8.5, incluido `git diff --check`. Rust `rustc 1.98.1 (48a229cea 2026-09-01)`, MinGW `x86_64-w64-mingw32-gcc (GCC) 13-win32`; fixtures compilados con `-O2 -Wall -Wextra -Werror`, Windows x86-64. Helper Rust release target `x86_64-pc-windows-gnu` enlazado con MinGW. No commit/push.

## Registro histórico del estado parcial

Las secciones siguientes documentan la implementación previa por pipe; el estado parcial y versión 0.8.4 son históricos, reemplazados por el cierre de arriba.

---

# Verificación LXMI 0.8.5 — Direct Inject Context Boundary

**Estado histórico previo a 0.8.5.1: PARCIAL / BLOCKED**, producto conserva versión **0.8.4**. No se declara cerrado ni se inicia 0.9.

## COMPROBADO — implementación

- Extensión del executor/context model existentes, sin segundo framework de procesos.
- Mismo target y DLL propios de 0.7, con modo target adicional para handshake de identidad; runner fijo `lxmi-context-inject-runner.exe`.
- Nonce fresco, staging por sesión, revalidación de firma/package/componente upstream y SHA-256 antes de copia.
- No se aceptan PID/target/DLL/path de frontend. Paths derivan del storage privado; backend rechaza Steam/compatdata/ZZZ y roots symlink u ownership incompatible.
- Protocol/nonce/PID del pipe/imagen del proceso y marker DLL correlacionados.
- Resultados, stdout/stderr, exit codes, mappings, candidatos wineserver y requisitos se mantienen separados.

## HOST TEST

Tool explícita: `Proton - Experimental`, `1790244902 experimental-11.0-20260924-x86_64`. No establece selección permanente del juego.

Primer run same-prefix nonce `77bf50a7c1c0d517d4ab799eec7861fb`: target PID Windows 220, runner 236; `Inject=0`, marker verified. Primer cross-prefix nonce `458c6cf8571442d0b05d957b59bc0153`: target PID 220, runner PID 220; filesystem visible; pipe inaccesible Win32 2; `Inject` no llamado. PID collision registrado como evidencia efímera.

El guard se reforzó para conservar un handle query-only **después** de identificar el target por pipe; la validación final vuelve a probar esos binarios. Los hashes y nonces finales se registran abajo al terminar los quality gates y Tauri.

## Native Tauri

Validación en `npm run tauri:dev`, aplicación `lxmi-desktop` nativa y controles reales mediante AT-SPI. No es browser Vite. Acciones: Scan Steam read-only, desplegar Context Lab, elegir Proton descubierto explícitamente, consentimiento del fixture, Same-Prefix y Cross-Prefix. Ambos resultados deben permanecer visibles en la misma ventana.

La UI explica que target identity unavailable impide llamar a upstream; no lo etiqueta como error observado de Inject. ZZZ readiness conserva evidencia/unknowns y gate cerrado. Warnings AT-SPI de otras aplicaciones Snap no se usaron como evidencia de LXMI.

## FIXTURE / offline

11 tests específicos nuevos: PID propio, nonce, mismatch del server PID, marker y éxito upstream independientes, filesystem no equivale a visibilidad target, requisitos no promovidos, same-context fake preserva package source, cross-context fake con PID collision no llama Inject, consentimiento, roots game/prefix rechazados. Fakes de executor y server observer evitan /proc, Proton y red en tests normales.

Suite normal: 247 passed, 0 failed, 10 ignored. Los tests host permanecen ignored y requieren `LXMI_DIRECT_CONTEXT_HOST_ACK=YES`, runtime explícito y modo opt-in; no fuerzan PASS cross-prefix ni lo interpretan como error upstream.

## Seguridad / filesystem

Baseline antes/después: size/mtime_ns/inode del EXE ZZZ, directory pfx4162040, system.reg, user.reg y config/config.vdf sin cambios. Esto cubre paths vigilados, no equivale a hash del árbol entero. Código copia únicamente del package verificado a un nuevo fixture; no reescribe managed runtime ni packages. No Hook, Steam launch, game process access, prefix del juego, debugger, custom injector o server kill.

## NO COMPROBADO / pendiente

- Llamada efectiva `Inject` cross-prefix bajo garantía de identidad LXMI.
- Requisito universal same-prefix/same-wineserver.
- Compatibilidad ZZMI/ZZZ y loader game context.
- Cierre/bump 0.8.5: bloqueado por el primer punto, incluso cuando todos los gates offline y el render nativo pasen.

## Resultados finales — 2026-10-01 UTC

### HOST TEST con binarios finales

| Caso | Nonce | PID target / runner (Windows, efímeros) | Resultado |
|---|---|---|---|
| Same-prefix | `eb7b83e4d7618dad6d51f26db516e49d` | 216 / 232 | `InjectedSuccessfully`; upstream 0; marker verificado |
| Cross-prefix | `ddd5f156396e87a0bb675b04e5ee8d6c` | 220 / 220 | `TargetIdentityUnavailable`; Win32 2 de pipe; API no llamada |

| Binario final | Tamaño | SHA-256 |
|---|---:|---|
| `lxmi-loader-test-target.exe` | 312764 | `18ce7d314ade05a565b5e8a5fdfe49a4c90e32eccefefe66521b4dc313eb20d9` |
| `lxmi-context-inject-runner.exe` | 308125 | `56c5f785e1427f09361dc5aff79485010540cc1949f6dfb6133d2a189741c6aa` |
| `lxmi-loader-test.dll` | 220839 | `a5e183a064096abf283f113105fd72865635b9beee07ab165d35f7b233d64007` |
| `3dmloader.dll` oficial | inventario del package revalidado | `44965ee51786db44fb4252671f356426ca7d879e6f7e8436b27d68e13237b0cf` |

Compilación fixtures con MinGW-w64 `x86_64-w64-mingw32-gcc`, PE x86-64; bridge helper Rust cross-build `x86_64-pc-windows-gnu` release también pasó. Los binarios oficiales no se añaden al repositorio.

### Native Tauri — COMPROBADO

Después del último rebuild se repitieron ambos botones en la misma ventana:

- Same-prefix nonce `2986363e0602ae14b72a58fc18923077`: PID target 220, runner 236; UI `injected_successfully`, marker + nonce PASS, upstream 0, mapping filesystem legible.
- Cross-prefix nonce `f72258d121c32fd33aca57efbebe0205`: PID target 220, runner 220; UI `target_identity_unavailable`, `BLOCKED`, `No invocada · Win32 2`; marker no verificado. Filesystem legible para ambos.
- Candidatos wineserver visibles en UI: same-prefix 811306 / 811306; cross-prefix 811634 / 811690. Se rotulan **UNKNOWN** para la relación exacta, no se eleva candidato a identidad.
- Ambos resultados permanecieron visibles simultáneamente. Las sesiones previas se reiniciaron durante hot rebuild, comportamiento esperado para snapshots efímeros; no se usaron como evidencia de retención tras reiniciar.
- Readiness evaluado mediante su botón declarativo: **NotReady**, gate `not_ready`, elegible para revisión no, consentimiento recibido no, lanzó juego no; selección de juego `Unknown`, estrategia sin decisión, Hook bloqueado y compatibilidad ZZZ no probada.
- La evaluación se hizo sin seleccionar una pareja de packages en ese panel: el servicio usa input nulo y deja `managed_runtime=Missing`. Esto no prueba que falte un runtime en disco. No se cambia ni se reensambla el runtime para esta revisión.

### Quality gates — COMPROBADO

Pasaron `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy workspace/all-targets/all-features con `-D warnings`, tests workspace (247 passed / 0 failed / 10 ignored), Windows build, frontend typecheck, lint, format:check, build y `git diff --check`. El detector mecánico del panel no reportó hallazgos. Los host tests se ejecutaron separadamente y con opt-in explícito.

### Cierre operacional

Se conservan los prefixes privados del test como artefactos LXMI; no se matan wineservers. El baseline de paths protegidos vuelve a comprobarse al cerrar la ventana. Solo se cierra Tauri/Vite propios; no se envía acción de cierre a ZZZ o Steam. Worktree cambia únicamente por este incremento, sin commit ni push automático.

Comprobación final posterior a cierre: metadatos protegidos sin cambios; no quedan procesos `lxmi-desktop` ni argv de target/runner de las sesiones `direct-context-v1` (comparación de argumentos completos, no substring del comando del scanner). No se enviaron señales a wineserver.
