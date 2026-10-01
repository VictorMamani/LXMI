# ADR-030 — Calificación de recuperación del prefix

Estado: aceptado para revisión del helper inocuo futuro, 2026-10-01.

## Contexto

MetadataOnly de 0.8.7 no satisface el backup requerido por GamePrefixEntryRisk. No se autoriza restauración sobre prefix real ni ejecución de Wine/Proton para probar recuperación.

## Decisión

Copia completa del árbol pfx mediante descriptors y no-follow; manifest íntegro, fuente estable y restore independiente a storage privado LXMI. Scope preserva contenido, estructura, enlaces, owner/group, mode, mtime y xattrs soportados. Se rechazan entradas/metadata no soportadas. Copias expanden hardlinks; no restauran inode/ctime, sparse topology, atime, estado fuera de pfx ni targets externos de symlinks.

Tras verificar nuevamente árbol backup, árbol restore, fuente actual, build/identidad/contexto y cobertura protegida, `RecoverableBackupVerifiedOffTarget` satisface la recuperación necesaria para `EligibleForReview` de la futura prueba inocua. La revisión no es autorización de ejecución. `FullRestoreVerifiedOnTarget` permanece no establecido y no existe executor real.

## Consecuencias

Manifest/hash/journal son evidencia de integridad local, no autenticación contra el propio UID. Sin freeze atómico de fuente: se usan inactivity checks e inventarios before/after. Contexto activo/cambios/tampering bloquean. No rollback automático ni restauración real; plan futuro exige nuevo consentimiento, contexto inactivo, identidad exacta y backup verificado. Efectos externos al scope requieren preflight adicional en el incremento futuro.

Hook guard, LoaderStrategyDecision NoDecision y gate de juego permanecen sin cambios. No se autoriza 0.8.8, 0.8.9 o 0.9 por este ADR. [Evidencia](../../01_Descubrimiento/preflight_prefix_zzz_0_8_7.md).
