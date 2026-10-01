# Verificación LXMI 0.8.7 — CERRADO mediante 0.8.7.1

## Registro histórico — primera revisión parcial

Fecha 2026-10-01. **NEEDS EVIDENCE.** Implementación de preflight y baseline comprobada; incremento no marcado cerrado/EligibleForReview. Versión permanece 0.8.6. Sin 0.8.8, 0.8.9 o 0.9.

## COMPROBADO — offline

Fixtures prueban hash del manifiesto, captura solo en root privado LXMI, MetadataOnly no desbloquea entry, actividad bloqueante, actividad desconocida, build distinto, registro faltante, rechazo de registro symlink y destino de baseline bajo game compatdata. Procfs sintético prueba prefix activo, entorno runtime denegado conservado como unknown y entorno ajeno denegado sin atribución falsa a Wine.

Quality gates: cargo fmt/check/clippy (`--all-targets --all-features -- -D warnings`), cargo test --workspace; frontend typecheck/lint/format:check/build; git diff --check. Resultado: **269 tests offline PASS, 11 ignored**. Tests host previos siguen ignored; no se ejecutó Proton, Wine, loader ni helper.

## READ-ONLY HOST — Tauri nativo

`npm run tauri:dev`, ventana lxmi-desktop. Se activó Scan Steam, Advanced → ZZZ Prefix Preflight, Consultar prefix preflight y Capture Prefix Baseline mediante accesibilidad de la ventana nativa. No Vite/browser como sustituto. Tauri/Vite se cerraron al terminar; no se enviaron señales a procesos Wine/ZZZ. Resultado visible: needs_evidence, same_build, UID/device/inode, actividad 0/0, inventario y limitaciones; recibo SHA-256 visible. No control de entrada real en este panel.

Evidencia efímera: `/tmp/lxmi-087-native-result.txt`, `/tmp/lxmi-087-after-comparison.json`. Manifest y digest exactos documentados en [descubrimiento](../01_Descubrimiento/game_prefix_entry_preflight_0_8_7.md). Comparación de 15 paths seleccionados: 0 diferencias observadas. No se afirma ausencia de cualquier escritura en trees no inventariados.

## Gate y límites

La policy de 0.8.6 requiere backup. La captura actual registra hashes/metadata y carece de copias restaurables. Rollback MetadataOnly; blocker rollback_insufficient_metadata_only. Se detiene antes de la fase real; no se solicita ni consume consentimiento de 0.8.8, no se ejecuta un helper dentro de 4162040. Ningún cambio de LoaderStrategyDecision o GameRuntimePlan.selected; el gate de juego conserva su cierre.

La implementación no restaura automáticamente registros. La siguiente revisión debe establecer cobertura de backup/recuperación suficiente y probarla con fixtures antes de valorar EligibleForReview. **UNKNOWN:** mutaciones exactas/reversibilidad de entrada real y compatibilidad ZZMI/ZZZ. El master prompt no se declara completado.

## Cierre 0.8.7.1 — COMPROBADO / HOST TEST

Fecha 2026-10-01. Producto **0.8.7**. Preflight **ELIGIBLE FOR REVIEW**, no ejecución autorizada. El blocker `rollback_insufficient_metadata_only` se resuelve con **RecoverableBackupVerifiedOffTarget**. No se inicia 0.8.8, 0.8.9 ni 0.9.

### READ-ONLY SOURCE

Prefix canonical `~/.local/share/Steam/steamapps/compatdata/4162040/pfx`: UID1000/device66306/inode11538172, identidad revalidada; juego/contexto asociado0 y runtime unresolved0 antes de backup y al reevaluar. Build `1790244902 experimental-11.0-20260924-x86_64`: SameBuild por metadata. Los checks no son un lock atómico. La consistencia completa de contenido/estructura/metadata soportada before/after no encontró diferencias; atime no pertenece a esa política. Paths protegidos selectivos sin diferencias observadas, sin reclamar todos los árboles Steam/juego.

### Backup VERIFIED / restore VERIFIED OFF-TARGET

Backup ID `f3551122468ed68b825a0182922f3be2`, bajo `~/.local/share/lxmi/backups/zenless-zone-zero/prefix/`. Manifest SHA-256 `db1d884b02ede5435f64520b92fcddf9e4688d56701c1329536d9abe38fa155f`, recalculado independientemente. 783 archivos, 285 directorios y 1377 enlaces =2445 entradas. Backup y restore: 2445 matching entries/783 hashes, 0 mismatches/unsupported metadata. Restore bajo `~/.local/share/lxmi/restore-tests/<backup-id>/tree/`, nunca prefix real; **NOT TESTED ON REAL PREFIX**. No executor real.

Fuente lógica434316729 bytes; backup asignado442859520 bytes. Libre antes62867013632, después de backup/restore61978664960; hubo builds concurrentes, no se atribuye toda la variación a backup. Gate 2x lógico +256MiB. Enlaces externos conservados como enlaces; no se copiaron destinos externos. La fuente host no tenía xattrs; su preservación se prueba con fixture sintético.

### Tauri nativo — COMPROBADO

Ventana nativa mediante AT-SPI: Scan Steam → Advanced ZZZ Prefix Preflight → consultar → Capture Prefix Baseline → Create Recovery Backup → Verify Backup → Test Restore in LXMI Sandbox → Re-evaluate Prefix Preflight. Resultado visible: `eligible_for_review`, recovery backup Verified, restore Verified off-target, real-prefix restore Not tested, entry no autorizada. No sustitución por browser/Vite. Se comprobó además reapertura/revalidación de evidencia persistida en la app0.8.7 sin crear otra copia. Tauri/Vite se cierran al terminar.

Evidencia temporal: `/tmp/lxmi-0871-native-result.txt`, `/tmp/lxmi-0871-native-reopened.txt`, `/tmp/lxmi-0871-host-summary.json`. Journal privado: `~/.local/share/lxmi/snapshots/recovery-f3551122468ed68b825a0182922f3be2.json`. Baseline inicial reevaluado `zzz-prefix-b19059f1671a9b680dcd53e4cb1b8388.json`, SHA-256 `e01d7b87ab67e6b104cedf49ff17be66d905b431d25a0ca58c8ed8458314b4b4`. La reapertura generó un baseline fresco `zzz-prefix-594c3adb21910164a8729f44b49753ff.json`, SHA-256 `00eb5975b17546006154e6fd6cb6f696a47a0be3b84e37f389e988a89bdbdea2`, conservando EligibleForReview.

### Calidad y regresión — COMPROBADO

Rust fmt/check/clippy all-targets/all-features -D warnings y cargo test workspace: **280 tests PASS, 11 ignored**, ninguno host/Proton ejecutado. Frontend typecheck/lint/format:check/build PASS; git diff --check PASS.

11 tests nuevos cubren árbol Wine sintético completo, enlaces dosdevices sin dereference, metadata/xattrs/mtime nanosegundo, tampering de backup/manifest/restore, fuente cambiante, actividad/identidad, FIFO/special files, paths/traversal/destinos symlink, espacio, staging cleanup y no reemplazo de IDs, transición de capability conservando otros blockers. Restore real permanece declarativo.

### Límites y decisión

[Política y resultados completos](../01_Descubrimiento/preflight_prefix_zzz_0_8_7.md) y [ADR-030](../04_Arquitectura_y_seguridad/ADR/030_prefix_recovery_qualification.md). La copia completa del scope soportado pfx, integridad y restore off-target constituyen evidencia suficiente **para revisión** de un helper inocuo posterior, no garantía de rollback real ni consentimiento para ejecutarlo. Prefix parent, targets externos, atime/ctime/birthtime/inodes, sparse y hardlink topology quedan fuera de restauración. Inmutabilidad por API, no protección contra el mismo usuario alterando archivos y evidencia conjuntamente.

LoaderStrategyDecision NoDecision; GameRuntimePlan.selected Unknown; gate de juego cerrado. **UNKNOWN:** comportamiento de una entrada real, mutaciones/reversibilidad fuera de scope y compatibilidad ZZMI/ZZZ. No Proton/Wine/ZZZ/helper/loader ejecutados, ninguna syscall de escritura dirigida a la fuente, ni restauración sobre compatdata real.
