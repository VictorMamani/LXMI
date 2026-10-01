# Verificación LXMI 0.8.7 — preflight con gate no alcanzado

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
