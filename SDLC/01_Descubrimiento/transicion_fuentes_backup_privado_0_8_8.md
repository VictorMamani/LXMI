# LXMI 0.8.8 — transición del estado histórico de fuentes en copias privadas

Fecha: 2026-10-03. Aplicación **0.8.7**. Continuación de diagnóstico autorizada después del resultado real `unexpected_mutation`. **0.8.8 BLOCKED**; sin nueva entrada real, restore real ni 0.8.9/0.9.

## Pregunta y diseño

La calificación anterior midió repeticiones sobre un prefix inicializado dentro del contenedor. El estado anterior al intento real conservaba fuentes modificadas por DirectProton. Este ensayo comprueba la transición desde ese estado concreto; no supone estabilidad para cualquier estado inicial.

Se reutilizan `QualifiedLinuxRuntimeRecipe`, el wrapper `run`, la copia privada verificada de Proton, el helper Windows sin modificaciones y el transporte privado consume-once. No se añade executor, comando Tauri, control frontend ni mecanismo de consentimiento.

Origen fijo: backup pre-entry `f4ea33c0c047d893319296420fd86784`, manifest SHA-256 `182929afbefb7e08b226cb7e1651a44a66df95e55e12422c58752bf81ed40f80`. Se verifica el backup completo antes y después. La identidad UID 1000/device 66306/inode 11538172 identifica su source histórico; **no se consulta ni usa el prefix real para ejecutar**.

Después de una calificación privada nueva del executor, se crean dos copias independientes mediante el motor descriptor-anchored existente. Ambas parten del mismo manifest completo y los mismos bytes de los tres hives. Se preservan symlinks como symlinks, sin seguirlos. Cada copia ejecuta dos solicitudes nuevas del helper. El timeout, allowlist, bindings y recibos del contenedor son los del executor existente.

## Evidencia y límites

Los reportes conservan hashes de hives, cambios por sección/campo, conteos de fuentes, metadata seleccionada, root identity y recibos del bridge. Los dumps completos de registro no se copian al repositorio; permanecen dentro de los árboles privados de LXMI. No se copian documentos externos a través de mappings.

El ámbito restaurado es `pfx`, no el compatdata padre ni sus locks/version/tracked files. Los inodes de las copias son nuevos. Se compara el estado antes/después dentro de cada copia; no se exige igualdad de inodes con el source histórico.

`InitialFontTransitionThenStable` es un hallazgo diagnóstico. No cambia `EnvironmentSpecific` a una categoría aceptada por el gate real. Campos desconocidos, diferencias entre réplicas o cambios repetidos se reportan explícitamente. La reserva one-shot real permanece consumida. Ningún resultado privado permite retry o consentimiento real.

## HOST TEST — resultado

**COMPROBADO:** nueva calificación privada del executor: seed más dos runs medidos correctos. Después, dos clones independientes verificaron el mismo inventario inicial completo `a4dc239a67b34726c5deab7f7bad38a0ae0b6ad8135b295bad5ac557d718c41a`. En cada clone se ejecutaron dos helpers, con nonce fresco, protocol/hash/exit correctos. Cuatro ejecuciones medidas, todas PASS.

| Copia | Run | Fuentes en cada una de las tres tablas | ContainerId | Otros cambios semánticos |
|---|---|---|---|---|
| 1 | 1 | 135 valores removidos, 368 rutas modificadas, cero añadidos | 4 GUID, `VolatileExpected` | Ninguna sección UNKNOWN |
| 1 | 2 | Estables | 4 GUID, `VolatileExpected` | Ninguna sección UNKNOWN |
| 2 | 1 | 135 valores removidos, 368 rutas modificadas, cero añadidos | 4 GUID, `VolatileExpected` | Ninguna sección UNKNOWN |
| 2 | 2 | Estables | 4 GUID, `VolatileExpected` | Ninguna sección UNKNOWN |

Los conteos describen valores de tablas duplicadas, no archivos independientes removidos del sistema. Las 368 rutas por tabla cambian de `Z:\usr\share\fonts` a `Z:\run\host`; en todos esos pares se conserva el filename. `system.reg` contiene dos tablas y por eso agrega 270 remociones/736 modificaciones; `user.reg` contiene la tercera. No se modificó manualmente el registro.

**COMPROBADO:** en ambas copias los hashes semánticos antes y después de **las tres secciones** son exactamente iguales a los del intento real fallido. Esto resuelve la falta de coincidencia exacta en dos secciones respecto al fixture genérico anterior. No se inventa un nuevo diff de valores del prefix real: la comparación usa el backup histórico y los recibos ya conservados del intento.

- `system.reg:Software\Microsoft\Windows NT\CurrentVersion\Fonts`
- `system.reg:Software\Microsoft\Windows\CurrentVersion\Fonts`
- `user.reg:Software\Wine\Fonts\External Fonts`

**DERIVED:** la diferencia de estado inicial explica la expectativa incorrecta de estabilidad: el self-test calificó repeticiones sobre un estado ya containerizado, mientras el source real conservaba el estado DirectProton. La transición histórica queda reproducida exactamente; no demuestra inocuidad funcional universal ni compatibilidad del juego.

NetworkCards, RunServices y RunServicesOnce no cambian en este ensayo. `userdef.reg` conserva contenido. La comparación seleccionada de metadata/mappings y la identidad raíz de cada clone pasan. En segundos runs los hives pueden cambiar bytes de timestamps de serialización sin cambios de valores fuera de ContainerId; esto se distingue de igualdad binaria completa.

## Guard añadido — COMPROBADO

La capacidad de sesión ahora conserva fingerprints de las **tres tablas iniciales** del self-test. `require_initial_prefix_state` exige evidencia completa y coincidencia exacta. Review la comprueba antes de generar un token de consentimiento y el executor vuelve a comprobarla después del preflight final, inmediatamente antes de invocar Proton. La ausencia, formato inválido o diferencia devuelve `initial_prefix_state_unqualified`.

No se añadió aceptación de grandes transiciones de fuentes ni wildcard de registro. La comprobación de fonts complementa los gates existentes de backup, fuente, protección, actividad y metadata; no representa equivalencia de todo el prefix. Un entorno calificado sin un estado inicial comparable no autoriza entry.

Este guard fue agregado después del ensayo privado y validado con tests sintéticos/check/clippy; **no se ejecutó el branch real para probarlo**. El diagnóstico privado usa el mismo argv/allowlist/transporte/pins; el guard nuevo aplica únicamente a Review/invocación real. El recibo de receta anterior al guard se conserva como tal, sin fingir que midió código agregado posteriormente.

## Artefactos

- [Reporte privado completo](../06_Verificacion/evidencias/private_font_transition_0_8_8.json), SHA-256 `3ea634ac3ee24669ffc6c8362e89ca31341d879f2954eff88746d89b9ad59340`.
- [Correlación con las tres secciones reales](../06_Verificacion/evidencias/private_font_transition_0_8_8_summary.json).
- [Tauri nativo, estado bloqueado conservado](../06_Verificacion/evidencias/private_font_transition_0_8_8_native.json).

Sesión de calificación privada: `5aaef5821ed992e4c672816ef7aa31ed`. Clones: `5fcc25869e2f1c45730ed12c95ee1a74` y `7857f354dd241e9fd1bc395dfab4a298`, bajo LXMI `tests/qualified-entry-v1`. El backup pre-entry permanece **VERIFIED / unchanged**, con manifest pin conservado y verificación completa antes/después. No se escribió sobre el backup ni su restore sandbox histórico.

## Validación y estado restante

**COMPROBADO:** fmt/check/clippy all-targets/all-features `-D warnings`; workspace **363 PASS / 0 failed / 12 ignored**; frontend typecheck/lint/format:check/build; `git diff --check`. Cinco tests nuevos cubren hallazgo inicial/repetición, UNKNOWN/réplicas distintas/evidencia incompleta, falta de calificación, estado inicial distinto y fingerprints completos medidos. Tests normales no usan Steam ni Proton real. Helper Windows sin modificaciones.

**COMPROBADO — Native Tauri:** después de recargar backend, Scan Steam y apertura del panel de historial; resultado `unexpected_mutation`, helper PASS previo y gates visibles. Review/Run permanecen deshabilitados. No qualification, consentimiento ni entry real desde Tauri en esta continuación. La versión 0.8.7 está comprobada en package/build; el header no fue visible en esta extracción AT-SPI y no se afirma validación visual de ese texto.

Árboles privados retenidos: aproximadamente 3.73 GiB allocated según `du` conjunto; espacio libre después 21,434,064,896 bytes. Esto es inventario de árboles retenidos, no una medición de coste incremental con accounting de todos los extents. Sin limpieza automática.

**BLOCKED:** el recibo real mantiene `unexpected_mutation`; su reserva one-shot sigue consumida e intacta. No nueva entrada, consentimiento, restore, lanzamiento de ZZZ, loader ni fases posteriores. Versión **0.8.7**, sin commit/push.

La siguiente decisión requiere revisar explícitamente qué estado inicial puede calificarse, y reconciliar en read-only el estado posterior con recuperación actual antes de plantear cualquier futuro experimento. La reproducción privada explica la transición; no convierte el intento fallido en PASS ni autoriza otro. La aceptación funcional de la transición histórica y el escenario con juego activo permanecen **NO COMPROBADO**.

## Corrección y calificación vinculada al source actual — 2026-10-03

La revisión nativa fresca del source posterior devuelve **UnderstoodAndAcceptable**: dos hives modificados y 35 diferencias de metadata frente al backup original, cero diferencias materiales sin resolver. Las tablas de fuentes ya no difieren semánticamente del estado original containerizado. La semántica Steam continúa ReviewedAndAcceptable/Stable, fingerprint `8677e768e0fc2a42985ea4c67a3cc20bc52b38900bb0a9affbfac6832cde0d70`. No se editó el registro para conseguir estos resultados.

**COMPROBADO:** el motor existente permite una nueva generación si la reconciliación fresca acepta el source, la generación anterior conserva integridad y ya no cubre el source actual. Si todavía lo cubre, exige reutilizarla. La historia conserva `f3551122468ed68b825a0182922f3be2` y `f4ea33c0c047d893319296420fd86784`; no se borran ni sobrescriben árboles o manifests. Los reportes históricos se muestran como evidencia registrada, sin afirmar revalidación fresca de cada árbol por el mero listado.

Nueva recuperación: `2b29b84bbc592fa091950bf046b3d668`, manifest SHA-256 `79685d4cdf3523d6acd5ab435a33a175438f488b871d13be3dd6cae0b8770597`. **VERIFIED**, 2445 entradas/783 archivos, source consistency/protected paths Verified; restore **VERIFIED OFF-TARGET**, sin diferencias ni metadata no soportada. Restore real **NOT TESTED**. Scope: pfx completo bajo la política existente, sin seguir symlinks; no compatdata padre, Steam state ni instalación del juego.

### La calificación ahora mide el estado inicial correcto

`qualify_integrated_executor_from_recovery` reutiliza la receta, copia privada de Proton, transport y executor existentes. Carga y verifica la generación actual y su restore; crea un clone completo verificado en LXMI storage. Captura fingerprints de las tres tablas **antes de la primera ejecución**, sin helper inicial para normalizar y ocultar la transición. Mide dos runs y vincula la capacidad a backup ID + manifest hash. El source real solo se lee para verificar aceptación/inactividad/protección antes y después.

Review exige esta vinculación y los fingerprints; un self-test genérico sigue disponible para investigación privada pero no puede autorizar entry real. Un cambio de generación, hash, fuentes, receta o evidencia incompleta bloquea. No se cambió el clasificador para aceptar fuentes EnvironmentSpecific ni campos UNKNOWN.

**HOST TEST / Tauri nativo:** sesión privada `ed785c1a68f15695671b1d4130a166b2`, desde la recuperación actual sin modificar su estado inicial.

| Evidencia | Run 1 | Run 2 |
|---|---|---|
| Helper protocol/nonce/marker hash/exit | PASS / 0 | PASS / 0 |
| Tiempo del helper | 2737 ms | 2029 ms |
| Fuentes | Estables, cero tablas modificadas | Estables, cero tablas modificadas |
| ContainerId | Cuatro campos GUID VolatileExpected | Cuatro campos GUID VolatileExpected |
| NetworkCards / RunServices / RunServicesOnce | Sin cambios semánticos | Sin cambios semánticos |
| Secciones UNKNOWN | 0 | 0 |
| Metadata seleccionada / identity privada / request cleanup | PASS | PASS |

Receta `6ff2ca659cd2db24d246c4cfa321fba1788203e4c956cebddbe161d50c841aed`, build aprobado sin sustitución, wrapper `run`, copia privada verificada. Reporte [source-bound qualification](../06_Verificacion/evidencias/current_source_executor_qualification_0_8_8.json), SHA-256 `06e6cb9c75cdefe54aca10a4f33a3a3b907709b065ca11c9d85959caf148e369`. La calificación genérica anterior no se relabeló como este resultado.

### Estado revisado y límite de autorización

La revisión final nativa devuelve **EligibleForReview**, UID1000/device66306/inode11538172, build SameBuild, contexto inactivo. Baseline fresco `zzz-prefix-64dac842206897b341916bed5974b11e`, SHA-256 `23bef28c6190fd6d0b5e163b4b476820f0b0aff9950922cd63b1bc39049e5c36`. [Resumen de la recuperación/calificación](../06_Verificacion/evidencias/current_source_requalification_0_8_8_summary.json) y [estado nativo final](../06_Verificacion/evidencias/current_source_readiness_0_8_8_native.json).

**COMPROBADO:** 365 tests PASS/12 ignored, fmt/check/clippy/frontend gates y diff check PASS. Native muestra Qualified y su recovery seed; permite calificar privadamente después del fallo, pero Review/Run reales siguen deshabilitados. Reserva y recibo `213181058a3f24c980a390dd5ab2797b` conservan sus hashes originales. No nueva entrada real, consentimiento, loader, restore ni fases posteriores.

Los blockers técnicos de recuperación y estado inicial quedan resueltos bajo la topología privada medida. **0.8.8 sigue abierto**: el experimento real histórico continúa UnexpectedMutation y su único intento está consumido. La regla FINAL del usuario exigía detenerse ante ese resultado. Para otra entrada se necesita autorizar explícitamente un experimento independiente, conservar la historia y después recoger nuevo consentimiento en Tauri. No se implementó bypass ni se borró una reserva para simular un primer intento.

Propuesta pendiente de autorización: un nuevo experimento inocuo, ligado al backup actual/receta/fingerprints; preflight inmediato, revisión y consentimiento nuevos; reserva independiente consume-once; una sola ejecución y misma política estricta before/after. Cualquier fallo detiene esa sesión sin retry/restore. No 0.8.9/0.9.

Árboles nuevos retenidos: sesión privada 2,728,607,744 bytes allocated; backup 443,932,672 y sandbox 442,884,096. Espacio libre al check final: 18,584,612,864 bytes. No limpieza automática. Son mediciones de árboles retenidos y espacio disponible, no atribución exclusiva de todos los cambios de uso del disco al experimento.
