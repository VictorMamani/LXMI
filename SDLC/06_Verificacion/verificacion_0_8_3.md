# LXMI 0.8.3 — Verificación de observación pasiva ZZZ

**Fecha:** 2026-09-30, hora local (sesión registrada en UTC el 2026-10-01).  
**Estado:** CERRADO.  
**Versión del producto:** `0.8.3`.

## Alcance y seguridad

La validación observó una ejecución normal de Zenless Zone Zero que el usuario inició mediante Steam. LXMI solo inspeccionó metadata procfs del UID actual y claves de entorno allowlisted. LXMI no inicia ni controla Steam/ZZZ/Proton/Wine, no carga XXMI ni loader, no modifica launch options, juego, compatdata o prefix, no lee memoria, no usa `ptrace`/debugger/tracing attach y no envía señales.

## Resultado manual en Tauri nativo

**LOCAL OBSERVED — evidencia proporcionada por el usuario desde la ventana Tauri y su terminal `npm run tauri:dev`:** La interacción manual se hizo antes del bump de versión; la terminal reportaba `lxmi-desktop@0.8.2` mientras ejercitaba el árbol candidato de observación 0.8.3. Después del bump, Tauri compiló y arrancó con metadata `0.8.3`.

- La sesión Tauri se inició a las `00:52:17Z`. El primer scan reportó `WaitingForGame`, 191 procesos del usuario y cero candidatos.
- A las `00:52:42Z`, el scan observó un proceso y registró 215 procesos, cuatro candidatos revisados, tres rechazos por nombre y un candidato aceptado. Scans posteriores conservaron un match.
- El usuario mostró la UI en `Observado · En curso · 67 s`: proceso `ZenlessZoneZero.exe`, PID `691954`.
- Identidad presentada: nombre esperado; entorno con AppID de Steam coincidente; entorno compatdata coincidente; ascendencia relacionada con Steam.
- **INFERIDO:** el primer scan vacío y el match posterior indican que el observer detectó ZZZ durante la ventana. No se afirma que el juego ya estuviera presente en el instante del primer scan. El fixture separado verifica el caso de proceso preexistente detectado en el primer scan.

## Evidencia de ejecución

| Dato | Resultado | Clasificación |
|---|---|---|
| Juego / proceso | `ZenlessZoneZero.exe`, PID `691954` | LOCAL OBSERVED |
| Steam AppID | `4162040`, verificado por entorno allowlisted | LOCAL OBSERVED |
| Mapping Steam por AppID | No encontrado en la fuente inspeccionada; default global presentado por separado | LOCAL OBSERVED |
| Proton de esta ejecución | `Proton - Experimental`, ID `1790244902`, build `experimental-11.0-20260924-x86_64` | LOCAL OBSERVED |
| Evidencia de asociación Proton | Path ejecutable/argv de proceso relacionado coincide con root de Proton instalado | LOCAL OBSERVED |
| Compatdata efectivo | `~/.local/share/Steam/steamapps/compatdata/4162040`, observado y coincidente con el esperado | LOCAL OBSERVED |
| Prefix efectivo | `~/.local/share/Steam/steamapps/compatdata/4162040/pfx/`, reportado como observado | LOCAL OBSERVED |
| Ascendencia | UI reporta ascendencia observada; Tauri registró un único match aceptado | LOCAL OBSERVED |
| Wineserver | Un proceso del UID actual expuso el mismo `WINEPREFIX` observado | LOCAL OBSERVED |
| Juego usa esa instancia de wineserver | No demostrado por un `WINEPREFIX` coincidente | UNKNOWN |
| Requisito del loader de compartir prefix/wineserver | No resuelto | UNKNOWN |

La tool observada corresponde a esta ejecución; no representa una selección permanente de Steam. Tools instaladas, runtime configurado y runtime observado continúan siendo datos distintos.

## Permisos de entorno y diagnóstico

La UI reportó `Entorno allowlist: Permiso denegado`, mientras el contador de la sesión mostraba cero lecturas denegadas entre candidatos directos. Esto no es contradictorio: el contador antiguo cubría procesos candidatos inspeccionados directamente; `environment_evidence` de un match también agrega el proceso observado y su ascendencia. **INFERIDO a partir del resultado UI y del flujo del observer:** al menos una lectura allowlisted de `environ` en el contexto de juego/ascendencia fue denegada, aunque las lecturas directas contabilizadas no lo fueran. La salida suministrada no identifica qué PID concreto recibió la denegación.

Se ajustó la UI y el modelo de contadores para distinguir candidatos directos de contextos aceptados cuya ascendencia tuvo entorno unavailable/permission-denied. Este ajuste explica la evidencia sin ocultar el estado de permiso.

## Readiness y límites

- `GameRuntimePlan.selected` permanece `Unknown`; el Proton observado no se copia a la selección configurada.
- `LoaderStrategyDecision` sigue `NoDecision`; Hook permanece bloqueado y Direct Inject sigue siendo únicamente evidencia del laboratorio LXMI.
- Readiness incorpora el summary de la sesión cuando se solicita su reevaluación; los tests verifican la integración y que no promueva la selección del planner. La salida manual compartida muestra la acción de reevaluar, pero no incluye un resultado posterior de esa acción.
- Steam/Linux/Proton + ZZMI, compatibilidad del juego, same-prefix/same-wineserver y loader readiness permanecen sin verificar. El gate continúa cerrado / `NeedsEvidence`.
- La captura se realizó con la observación todavía activa a los 67 segundos. El estado terminal posterior no se incluyó en la evidencia compartida; su valor es **UNKNOWN**. La sesión está acotada a 90 segundos.
- El código de observación es read-only y no expone comandos de escritura/control. No se tomó un hash pre/post del árbol del juego, Steam o prefix, así que no se afirma una comparación independiente de filesystem.

## Implementación comprobada por tests

- Observer Linux procfs con filtro de UID actual, `starttime` revalidado, límites por archivo y ancestry limitada.
- Solo se leen `STEAM_COMPAT_APP_ID`, `SteamAppId`, `SteamGameId`, `STEAM_COMPAT_DATA_PATH` y `WINEPREFIX`; los resultados son efímeros y paths de home se redactan.
- Identidad combina nombre del ejecutable esperado con evidencia independiente; matches múltiples permanecen ambiguos.
- Scan inmediato permite detectar procesos ya existentes; lectura de `cmdline` puede identificar el juego aunque `/proc/<pid>/exe` sea un preloader de Wine.
- `PermissionDenied` se distingue de `Unavailable`. Un fixture comprueba que un ancestro inaccesible se informa separado de los contadores de candidatos directos.
- DTO Tauri serializa snapshot/diagnósticos, los estados terminales son explícitos y la UI conserva el último resultado visible fuera del disclosure plegable.
- Readiness consume la observación sin mutar `GameRuntimePlan.selected`, la estrategia loader o el gate.

## Quality gates

Resultados después del ajuste de la diferencia entre entorno directo y ascendencia:

| Comando | Resultado |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo test --workspace --quiet` | PASS — 221 passed, 8 ignored (host/network/opt-in) |
| `npm run typecheck` | PASS |
| `npm run lint` | PASS |
| `npm run format:check` | PASS |
| `npm run build` | PASS |
| `git diff --check` | PASS |
| `npm run tauri:dev` | PASS — el usuario hizo la observación en una ventana Tauri nativa antes del bump; tras actualizar metadata, Tauri `0.8.3` compiló y arrancó. El ajuste aclaratorio de contadores posterior pasó build/tests; no se repitió una sesión ZZZ para esa copia aclaratoria |

El proceso nativo iniciado para validar 0.8.3 se cerró con `Ctrl-C`; `ps -C lxmi-desktop` no mostró proceso y `lsof -nP -iTCP:1420 -sTCP:LISTEN` no reportó listener. La sesión observada se mostró activa a los 67 s; no se recibió el snapshot terminal posterior al límite de 90 s.

## Cierre

La observación real hizo visible la causa que el prototipo anterior ocultaba: la UI previa colocaba el resultado dentro de un disclosure plegable y carecía de diagnóstico por scan; la sesión histórica no permitía decidir si el backend había observado ZZZ. La sesión actual produjo un match real y mostró su evidencia. Se actualizó la versión a `0.8.3`. No se inició LXMI 0.8.4/0.9, ni hubo launch/injection.
