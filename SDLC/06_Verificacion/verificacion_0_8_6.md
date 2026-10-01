# Verificación LXMI 0.8.6 — Existing Wine Context Re-entry

**COMPROBADO: CERRADO para fixture.** Producto 0.8.6. Same-prefix late reentry VERIFIED; control different-prefix no verificado. No loader ejecutado, no Inject, no game-prefix entry, ningún incremento 0.9 iniciado.

## Implementación

- Coordinador/staging/argv executor de 0.8.5.1 reutilizados; no segundo sistema de ejecución Proton.
- Target existente en modo reentry publica identidad completa y crea evento Local con nonce; conserva sus modos previos.
- Helper compilado con LXMI_REENTRY_ONLY excluye carga upstream/Inject. Solo dos PE propios staged, runtime sin DLL payload, upstream null.
- Nuevo DTO ContextReentryResult separa prefix relation, evento, binding, filesystem, evidencia endpoint y riesgo declarativo.
- A publica ready antes de B; delay 300 ms y worker activo. B usa una segunda llamada directa argv independiente, no child Windows de A. No pipe requerido para startup/binding.
- Query exclusiva del PID fixture propio con PROCESS_QUERY_LIMITED_INFORMATION; PID/image/FILETIME/nonce siguen obligatorios. No enumeration, VM rights, ptrace o memoria.
- Root private LXMI y guards Steam/game/compatdata4162040; no parámetros arbitrary PID/path/DLL. Tool seleccionada debe pertenecer al discovery local.

## HOST TEST — CLI opt-in

Tool explícita: Proton Experimental 1790244902 experimental-11.0-20260924-x86_64; no selección configurada del juego.

| Caso | Nonce | Resultado |
|---|---|---|
| Same-prefix | db572a81dd935ebb124d3f81e49f31c3 | Verified; evento visible/señal recibida, identidad exacta, prefix dev/inode coinciden |
| Different-prefix | bc635e87cb78d4de17da32ce150f79c9 | PrefixMismatch; evento no visible; PID collision: imagen consultada es helper B, creación distinta |

Ambos leen marker filesystem, no implica context join. Inyección no llamada ni helper capaz de ejecutarla. Sockets derivados observados; PID de wineserver/conexión exacta UNKNOWN. Tras salida, sockets desaparecieron en lectura posterior sin señales/cleanup wineserver. [Investigación con fuentes fijadas y mutaciones](../01_Descubrimiento/wine_context_reentry_0_8_6.md).

Comando opt-in usa LXMI_REENTRY_HOST_ACK=YES y LXMI_HOST_PROTON_SCRIPT explícito; LXMI_REENTRY_MODE=cross para el control. Test existing_context_reentry ignorado por defecto. No Steam URLs/applaunch, Wine standalone ni game process.

## Native Tauri — COMPROBADO

Aplicación nativa lxmi-desktop mediante npm run tauri:dev. Controles reales AT-SPI: Scan Steam read-only, Advanced Context Lab, Proton explícito, consentimiento reentrada sin loader, Run Same-Prefix Re-entry, Run Different-Prefix Control, readiness. No browser-only; no se atribuye esta automatización al usuario como sesión manual humana.

Ambos resultados permanecen visibles: same-prefix verified y control prefix_mismatch; evento y metadata query independientes, canonical prefixes/dev/inodes y endpoints candidatos con rótulo DERIVED. Wineserver conexión exacta UNKNOWN; Inject no llamado. Readiness renderiza fixture reentry verified; game-prefix entry no probado, loader UNKNOWN, gate cerrado, GameRuntimePlan.selected Unknown. Evaluación sin package input seleccionado mantiene NotReady; no prueba ausencia del managed runtime en disco.

### Repetición con producto 0.8.6

| Caso | Nonce de UI nativa | Evidencia final |
|---|---|---|
| Same-prefix | `23fc8bb288c4e39eef91080da0588346` | Re-entry verified; FILETIME esperado/consultado 134353041164425240; evento visible y señal recibida; dev/inode 66306/12467965; socket candidato observado |
| Different-prefix | `ab3585dfc797b42d9685e31720b7160b` | PrefixMismatch; PidNotVisible Win32 87; evento no visible; dev/inodes 66306/12480259 y 66306/12486097 |

Esta repetición no mostró colisión: el PID no era visible desde B. La colisión permanece evidencia del CLI anterior, no se fuerza como resultado. Socket candidato A observado y B no observado en ese instante: se conserva Unknown para el endpoint efectivo de B. Ambos markers filesystem legibles; Inject no llamado. Resultados persistieron simultáneamente; readiness muestra fixture verified y game entry NotTested. Tauri/Vite propios cerrados después de revisar, sin acciones sobre Steam/juegos/server.

## FIXTURE / offline

263 passed, 0 failed, 11 ignored. Nueve tests adicionales al checkpoint: comparación dev/inode independiente del path textual; target preexistente/late helper; different-prefix; filesystem-only no promueve reentrada; proceso salido/creation mismatch; riesgo declarativo cerrado; fake coordinator exige ready antes de helper y no carga DLLs; paths reales fixture que contienen Steam/4162040/ZZZ rechazados antes de ejecutar. Fake target que sale antes de ready impide iniciar el segundo helper. Existing nonce/PID/image/time, bridge, loader, observer y context tests siguen pasando. Tests normales sin Steam/Proton/red/real procfs.

## Toolchain / artefactos

Rust 1.98.1; MinGW-w64 GCC 13-win32, flags -O2 -Wall -Wextra -Werror, target PE x86-64. `tools/lxmi-loader-lab/build.sh` compila target y helper reentry además de artefactos anteriores; compilar los anteriores no los ejecuta. Helper bridge Rust cross-build release x86_64-pc-windows-gnu también pasó.

| Binario del probe | SHA-256 |
|---|---|
| lxmi-loader-test-target.exe | 1b890337ab1173518f24287fa9437784f89d303730c4cb4adfe1fd4585505da6 |
| lxmi-context-reentry-helper.exe | e537fbfc164563415c0f16118740729b1d30bdbf29a8241125d8f0d102a9ebc9 |

Hashes validados durante copia, inventory por sesión fuera de runtime. No upstream binary en repo ni managed ZZMI mutado.

## Quality — COMPROBADO

cargo fmt --all -- --check, cargo check --workspace, cargo clippy --workspace --all-targets --all-features -- -D warnings, cargo test --workspace; Windows MinGW fixture builds y Rust bridge build; frontend typecheck/lint/format:check/build; git diff --check. Detector mecánico del panel: sin hallazgos. Gates repetidos con producto 0.8.6.

## Seguridad / cleanup

Baseline size/mtime_ns/inode de EXE ZZZ, pfx4162040 directory, system.reg/user.reg y config.vdf: sin cambios. No hash de todo el juego/prefix. Packages/runtime ZZMI no reciben writes; únicamente nuevo test storage y sockets/locks Wine temporales. Prefix snapshot ligero muestra mutaciones normales posibles; no autoriza game-prefix entry ni prueba rollback.

Target sale por stop marker propio/timeout, helper sale por respuesta. Executor gestiona solo sus launcher children; no kill global, server control o process discovery. Tauri/Vite propios se cierran tras revisión. Se deja a Wine limpiar naturalmente; endpoint/PID exacto sigue UNKNOWN.

## UNKNOWN / NO COMPROBADO

Reentrada al prefix ZZZ, requisitos loader/game, compatibilidad ZZMI/ZZZ y reversibilidad completa. Alias symlink y diferente Proton no probados. Subtest late Direct Inject opcional no ejecutado; este incremento permanece sin loader. Strategy NoDecision, Hook guard intacto y gate del juego cerrado.

## Git

main, checkpoint c5c71e1 preservado; worktree inicial limpio. Cambios de 0.8.6 sin commit ni push automático.
