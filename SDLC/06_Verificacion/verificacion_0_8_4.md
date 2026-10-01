# LXMI 0.8.4 — Verificación Context Join

Fecha: 2026-09-30 local / 2026-10-01 UTC. Estado: CERRADO para laboratorio fixture; gate de juego cerrado. Producto `0.8.4`.

## Alcance

Se ejecutaron únicamente dos PE propios mediante Proton Experimental explícito en directories nuevos de LXMI. Ningún juego, loader, XXMI, Hook o Direct Inject fue iniciado. No se usó/copió `compatdata/4162040`, ni el runtime ZZMI. Investigación, hashes y fuentes fijadas: [contexto Wine/Proton](../01_Descubrimiento/contexto_wine_proton_0_8_4.md).

## HOST TEST — resultados

| Caso | Marker A/B | Evento A/B | Prefix | Mappings completos | Wineserver |
|---|---|---|---|---|---|
| Mismo prefix | legible / legible | señalado / abierto | mismo, reportado por peers | mismos | candidato común; identidad UNKNOWN |
| Prefixes distintos | legible / legible | timeout propio / no encontrado (Win32 2) | distintos, reportados por peers | C distintos; Z común | candidatos separados; identidad UNKNOWN |

Los peers coexistieron: A publicó un ready marker después de crear el evento y conservó su handle hasta recibir la señal de B o expirar 12 s. B se inició solo tras leer ese ready marker con nonce. El control mantuvo la misma tool y el mismo nonce entre A/B. No se tomó igualdad de entorno como prueba del evento; el probe requirió OpenEvent/SetEvent y recepción en A.

Las suites host explícitas se ejecutaron con `LXMI_RUN_CONTEXT_HOST_TEST=YES` y paths privados. Los tests normales dejan esta suite ignorada. La función mantiene el resultado observado; no impone que Wine de otra versión reproduzca el mismo resultado.

## COMPROBADO — ventana nativa Tauri

Se abrió `npm run tauri:dev`, proceso `lxmi-desktop` real. Mediante AT-SPI se recorrieron sus controles:

1. Scan Steam.
2. Abrir Advanced / Proton-Wine Context Lab.
3. Consultar y seleccionar el Proton Experimental descubierto, confirmar creación de prefixes privados.
4. Ejecutar mismo prefix; revisar resultado retenido.
5. Ejecutar prefixes distintos; revisar resultado separado.
6. Abrir Launch Readiness de ZZZ y solicitar únicamente su evaluación read-only.

El árbol/texto accesible nativo mostró `Evento compartido: sí/no`, PIDs Windows de cada peer, marker legible, estado del evento, prefixes, rutas Windows y candidatos wineserver. En la versión final se distinguieron `drive_mapping: different` de `marker_path_mapping: same` en el control. Los dos resultados permanecieron simultáneamente en el estado React. No se usa una captura browser Vite como evidencia Tauri. La interacción se automatizó sobre la ventana nativa por accesibilidad; no se atribuye una revisión visual manual humana al usuario. Una captura de pantalla no se utilizó como evidencia.

Los fixtures finales del recorrido nativo usaron Windows PID 220/236 y wineserver candidato 752427 (mismo prefix); PID 216/220 y candidatos 753602/753653 (control). Estos PIDs no son identidad persistente. El panel informa wineserver `unknown`, incluso con un candidato común.

Readiness mostró Proton del juego `Unknown`, requisitos same-prefix/context sin resolver y Hook bloqueado. El modelo de estrategia sigue `NoDecision`; este incremento no resuelve compatibility ni abre el gate.

La interacción funcional se hizo con metadata de producto `0.8.3` sobre el árbol candidato 0.8.4. Después se actualizó metadata a `0.8.4`: Tauri compiló/arrancó y se identificó su aplicación nativa en AT-SPI. La búsqueda automatizada de la etiqueta de versión dentro de interfaces Text no encontró esa cadena; no se afirma que su etiqueta visible haya sido comprobada independientemente. Los manifests y el build sí reportan 0.8.4. Ninguna observación de ZZZ se inició en este recorrido.

## COMPROBADO — límite filesystem

Se compararon tamaño, mtime_ns e inode pre/post del ejecutable `ZenlessZoneZero.exe`, directory `4162040/pfx`, `system.reg`, `user.reg` y `config/config.vdf`: sin cambios en esos paths. No se hasheó el árbol completo ni se afirma una auditoría integral de Steam. Antes del lab no se identificó un ZZZ ejecutándose en metadata del UID actual.

Las escrituras previstas fueron bins/hash inventario de test, nonce markers y prefixes privados en `lxmi/tests/context-join-v1/`. HOME y XDG del proceso se aislaron allí; Wine usa también sockets/locks propios en `/tmp/.wine-<uid>`. No se modifica el original de ningún paquete/runtime. Los peers tienen salida propia acotada; no se señaliza wineserver. En timeout, Context Lab solo puede terminar el launcher Linux hijo; el executor captura pipes en modo no bloqueante y conserva límites de salida. No se usan shell, process memory, ptrace, OpenProcess, scanners Windows, DLLs propias o upstream ni red desde peers.

## FIXTURE — cobertura offline

Se añaden 15 tests enfocados: relaciones independientes y estados unknown/conflicting; mismo/diferente prefix con fake executor y dosdevices sintético; nonce/protocol/response; output truncado/nonzero; consentimiento antes de ejecución; symlink/hash alterado; root externo/game rechazado; game join declarativo sin autorización; política de timeout sin grupos y captura acotada. Los tests de fake peers no leen /proc real: el observador de wineserver se inyecta. Los tests anteriores de bridge, Loader Lab, observation y readiness se mantienen.

## Quality gates finales

| Validación | Resultado |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo check --workspace | PASS |
| cargo clippy --workspace --all-targets --all-features -- -D warnings | PASS |
| cargo test --workspace | PASS — 236 passed, 9 ignored |
| Windows peers, MinGW | PASS — PE32+ x86-64, hashes e inventario registrados |
| npm run typecheck / lint / format:check / build | PASS |
| git diff --check | PASS |
| Tauri nativo | PASS — controles reales ejercitados por accesibilidad; sin browser-only sustituto |

No se ejecutaron tests host de loader/Hook/ZZZ. Los ignored de red/game/runtime ajenos al Context Lab permanecieron sin ejecutar.

## UNKNOWN y próximos requisitos

- Endpoint/instancia wineserver exacto de cada peer, y del ZZZ observado en 0.8.3.
- Necesidad del loader futuro de compartir tool/prefix/wineserver/sesión.
- Compatibilidad ZZMI/ZZZ, incluso si hay visibilidad de objetos entre fixtures.
- Mutaciones que produciría un helper al entrar al prefix real de juego.

`GameContextJoinPlan` carece de executor y declara juego no autorizado. Se requiere primero un incremento específico de riesgo/preflight si la arquitectura futura propone usar compatdata del juego. No se inició 0.9.

## Cleanup final

Se cerraron las dos sesiones Tauri/Vite iniciadas por el agente con Ctrl-C. Tras los experimentos no se identificaron peers LXMI aún vivos ni ZZZ. No se señalizó ningún wineserver; cualquier servidor privado residual se deja salir naturalmente. La comparación final volvió a dar metadata protegida sin cambios. No se realizó commit ni push.
