# LXMI 0.8.8.3 — Recovery Source Reconciliation

Fecha: 2026-10-02. Versión **0.8.7**. **0.8.8 BLOCKED / PARTIAL**. No helper, Proton/Wine, juego, loader ni restore contra el prefix real; no consentimiento. No 0.8.9/0.9.

## Implementación

Se amplía el scanner de recuperación descriptor-anchored existente: árbol completo, O_NOFOLLOW/O_NOATIME, tipos/mode/UID/GID/mtime/hash/xattrs/targets de symlink, mismos límites y rechazo de special files/mounts/owners no soportados. No segundo motor de snapshots. Dos scans con stamps/identidad y una comprobación adicional de fingerprint detectan inconsistencia; activity same-UID revalidada antes/después. No son un lock atómico.

`RecoverySourceDriftReport` conserva diferencias Added/Removed/Modified/TypeChanged/SymlinkTargetChanged/MetadataOnlyChanged, estado anterior/actual por entrada, clasificación/evidencia y correlación de registros. El JSON completo se guarda en snapshots LXMI con nombre nuevo/0600 y se copia como artefacto SDLC; la UI solo resume categorías, hives y paths protegidos. No raw registry/config values, environment ni PIDs persistidos.

`CurrentPrefixStateAssessment` controla los dos actions separados de baseline aceptado/nueva generación. Solo `UnderstoodAndAcceptable` permite avanzar; backend revalida freshness, fingerprint completo, identidad, build, inactividad, protected evidence y backup histórico. No se acepta un assessment del frontend. Las consultas read-only ya no generan baselines automáticamente; una captura anterior se reutiliza únicamente si su evidencia seleccionada permanece idéntica.

Crear recuperación actual usa el mecanismo verificado previo, ID nuevo/staging exclusivo, consistencia de fuente, verificación y protected before/after. La restauración continúa limitada al sandbox LXMI. El backup histórico nunca se reemplaza ni borra. El mismatch antiguo requiere reconciliación; no se exige igualdad con el backup histórico para siempre.

## READ-ONLY HOST

Prefix canonical `/home/university/.local/share/Steam/steamapps/compatdata/4162040/pfx`, UID **1000**, device **66306**, inode **11538172**, sin cambios. Contexto: **0 matching / 0 unresolved** en comprobaciones puntuales. Build `1790244902 experimental-11.0-20260924-x86_64`, **SameBuild**. Manifest del árbol fuente Proton instalado coincide con el fijado por la receta: `9d0bb73e241bac979afeda526b589fce6f26ee2d5e8822b7aadc710b179421bb`; lectura completa, sin ejecutar Proton.

Backup histórico `f3551122468ed68b825a0182922f3be2`: **VERIFIED**, 2445 entradas/783 file hashes, sin mismatches. Manifest SHA `db1d884b02ede5435f64520b92fcddf9e4688d56701c1329536d9abe38fa155f`; el sidecar conserva ese mismo valor. Restore original re-verificado **VERIFIED OFF-TARGET**, sin crear otra restauración ni restaurar el prefix real. Su integridad no depende de igualdad con el source actual.

Baseline 0.8.7 utilizado: `zzz-prefix-594c3adb21910164a8729f44b49753ff.json`, SHA `00eb5975b17546006154e6fd6cb6f696a47a0be3b84e37f389e988a89bdbdea2`, verificado antes de comparar. No se sustituye por una captura actual para ocultar drift.

## Diff completo del prefix

| Categoría | Entradas |
| --- | ---: |
| Added | 0 |
| Removed | 0 |
| Modified (contenido) | 2 |
| TypeChanged | 0 |
| SymlinkTargetChanged | 0 |
| MetadataOnlyChanged | 35 |
| Material UNKNOWN / Unexpected | 0 |

Contenido modificado únicamente en `system.reg` y `user.reg`. Metadata-only en raíz `.`, `dosdevices`, 32 symlinks `dosdevices/com1` a `com32` y `userdef.reg`. Mismo tipo, contenido/size/target, mode, owner y xattrs; solo mtime difiere. No se siguieron los targets COM ni Z:. Writer exacto de timestamps **UNKNOWN**; no se presenta como cambios de contenido, permisos o topología.

Fuente consistente durante scans/fingerprint: **VERIFIED**. Artefacto completo: [recovery_source_drift_0_8_8_3.json](../06_Verificacion/evidencias/recovery_source_drift_0_8_8_3.json), SHA `7b286dea15af063741ff4b6c5235838ac0dfe2bbda94e4f8fbdfa803c646a098`. Receipt original privado `recovery-source-drift-76e00dd8d6161d99664e271e26ee3537.json`; el hash del receipt se muestra en Tauri.

## Correlación histórica y clasificación registry

**COMPROBADO:** bytes del backup coinciden con hashes del before del intento DirectProton `699f4768f21d30842653eb404baeb237`; bytes actuales coinciden con su after en los tres hives. Se verifica la integridad de ambos receipts, no solo parecido entre rutas. Estado actual de fuentes conserva exactamente las mutaciones históricas; no se normalizó ni revirtió.

| Hive / grupo | Drift actual vs backup | Clasificación |
| --- | --- | --- |
| system.reg: dos tablas Fonts | +135 valores / 368 paths reescritos por tabla | Historical ENVIRONMENT SPECIFIC; KnownWineProtonMutation por correlación exacta |
| user.reg: External Fonts | +135 / 368 | Historical ENVIRONMENT SPECIFIC; misma correlación |
| Cuatro device ContainerId | GUID-only en los cuatro dispositivos fijados | PreviouslyQualifiedVolatile / reglas exactas VolatileExpected |
| NetworkCards parent vacío | Omisión del hive guardado | StableExpected / DERIVED según serializer; no prueba de eliminación de clave viva |
| Wow6432Node RunServices / RunServicesOnce | **Sin diferencias** en el source real | No se aplicó regla de creación vacía; fue observado solo en fixtures |
| userdef.reg | Contenido idéntico | MetadataOnly; ninguna mutación semántica |

Secciones UNKNOWN: **0**. Los conteos Fonts son por tabla, no número de archivos físicos únicos. La causalidad de componentes Wine/font mounts permanece **DERIVED** a partir de fuentes upstream fijadas y matriz privada 0.8.8.1; los hashes prueban la identidad del estado almacenado, no trazan al writer de cada llamada.

La matriz DirectProton privada reprodujo esos grupos; ambas recetas container calificadas y el executor integrado 0.8.8.2 mantuvieron fonts estables. Sus resultados no autorizan aceptar cambios reales futuros fuera de reglas calificadas. Véase [entorno Linux](entorno_linux_fixture_0_8_8.md). No se ejecutó otra matriz ni se otorgó una capacidad nueva de executor en esta sesión de reconciliación read-only; la evidencia previa Qualified continúa histórica, sin regresión observada de receta/build/fuente.

El primer intento bloqueado por protected_path_changed conserva selected_prefix_paths idénticos entre review y preflight, antes de invocar helper. No explica el drift de fuentes posterior. Los cuatro receipts anteriores se releyeron y verificaron en [recovery_history_correlation_0_8_8_3.json](../06_Verificacion/evidencias/recovery_history_correlation_0_8_8_3.json).

## Protected state

De 9 registros protegidos de la policy 0.8.7, **2 difieren**:

| Path | Size antes / actual | SHA antes / actual |
| --- | --- | --- |
| Steam/config/config.vdf | 36642 / 29573 | `4a62023de274ccfc3f7df1ecb5e7ed1b8bb4b71b07516d31ddb84ec108920d18` / `b69ced69a04f3a2fe810a5428642941e53ccd9f47b8e1347f7a0195eaa3bd1c7` |
| Steam/userdata/1416423935/config/localconfig.vdf | 59838 / 59838 | `bb1a326852358100d2fe652ac203e693ec992f016f98e5311096df57af0a9eeb` / `a3570c17697303b91924405d72361f52f0110ec2eda8b8328e0b8717b0da68f4` |

Inode/mtime también difieren; metadata completa está en el diff. Mtime actual UTC: 2026-10-01 11:05:27 y 11:05:28, respectivamente.

**COMPROBADO:** ambos hashes actuales ya aparecen iguales en el before y after del intento DirectProton del 2026-10-02. Por tanto esas diferencias protegidas no se introdujeron entre aquellos snapshots ni en esta reconciliación read-only. **UNKNOWN:** autor, claves modificadas, semántica y launch options históricas; el baseline 0.8.7 solo conserva hashes/metadata, no contenidos Steam que permitan reconstruir un diff de claves. No se los denomina KnownSteamMutation ni benignos por esa razón.

Los otros 7 registros protegidos coinciden con 0.8.7: ejecutable ZZZ, game root/userdata seleccionados, appmanifest, otro localconfig, packages y runtimes según la cobertura limitada. La fuente Proton coincide con su manifest calificado. No se afirma hash completo de game/packages/runtimes ni ausencia de toda escritura externa.

## Decisión y STOP

**CurrentPrefixStateAssessment: ProtectedStateChanged.** Prefix drift material entendido, pero estado protegido no aceptado. Blocker preciso: `protected_state_needs_review`; preflight **NEEDS EVIDENCE**, recuperación actual no promovida. El backup histórico permanece válido; no es corrupción del backup.

No baseline nuevo, nueva recuperación, nuevo restore sandbox, consentimiento ni entry real. El backup original y baseline antiguo permanecen preservados. Real-prefix restore **NOT TESTED**. La política permite futuras generaciones cuando source y protección sean realmente aceptados; no hay override para conseguir PASS. 0.8.8 sigue abierto en 0.8.7; no 0.8.9/0.9.

## Validación

**COMPROBADO — Native Tauri:** Scan Steam, Advanced Prefix Preflight, Reconcile Recovery Source — Read Only, diff/assessment/historia visibles. Botones Capture Fresh Accepted Baseline, Create Current Recovery Backup, Review y Run reales disabled/sensitive=false. [Recibo nativo](../06_Verificacion/evidencias/recovery_reconciliation_0_8_8_3_native.json). No browser-only, checkbox ni Run. Tauri/Vite propios cerrados; no control de Steam/Wine/juego.

Tests offline: **340 PASS / 0 failed / 12 ignored**. Incluyen categorías de diff, cambio de symlink sin dereference, metadata y permisos, GUID/font con correlación estricta, unknown/protected/activity/identity/source inconsistency bloqueantes y generación sintética nueva con backup/baseline antiguos preservados, restore off-target y preflight EligibleForReview del fixture. Tests normales no inspeccionan Steam/procfs reales ni ejecutan Proton.
