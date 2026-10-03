# Verificación LXMI 0.8.8 — BLOCKED / UnexpectedMutation

Producto permanece0.8.7. **HOST TEST:** el último intento consentido ejecutó el helper con protocol1/nonce/hash/exit0. **BLOCKED / UnexpectedMutation:** diferencias registry/metadata no aceptadas; protected paths sin diferencias dentro de cobertura. El intento previo ProtectedPathChanged y sus comprobaciones se mantienen como historia abajo. No0.8.9/0.9 ni restore automático.

## Historia del primer intento y diagnóstico read-only

**COMPROBADO:** implementación offline de entry inocuo y consentimiento one-shot, bridge original fixture-only conservado. En el primer intento el usuario consintió y pulsó el botón; el preflight inmediato bloqueó por ProtectedPathChanged antes de ejecutar helper. No se aceptó consentimiento en su nombre ni se reintentó automáticamente.

Quality gates: Rust fmt/check/clippy all-targets/all-features -D warnings; cargo test workspace **285 PASS /11 ignored**; frontend typecheck/lint/format:check/build y git diff --check PASS. Windows helper no modificado, no cross-build necesario. Tests host previos ignorados.

5 tests nuevos: capability interna/mismo protocolo y argv fijo sin group cleanup; generic bridge rechaza nuevo modo real; mutations registry hashes/metadata distinguidas; protected/mapping changes bloqueados; resultados de fallo jamás abren gate; consentimiento/actividad rechazados antes de staging/execution. Fixtures offline únicamente.

Native Tauri COMPROBADO mediante AT-SPI: Scan Steam, Advanced ZZZ Prefix Preflight y Review Harmless Prefix Entry. Resultado EligibleForReview, inactividad0/unresolved0, SameBuild y backup f3551122468ed68b825a0182922f3be2 Verified. Consent checkbox comprobada sin aceptar; helper Not run. Evidencia `/tmp/lxmi-088-native-review.txt`. El usuario posteriormente aceptó el checkbox y pulsó el botón. Resultado nativo `protected_path_changed`; el helper no llegó a ejecutarse. Nonce/hash/exit real siguen NO COMPROBADO. No se inicia0.8.9/0.9, no version bump/commit de fase cerrada.

[Diseño, fuente y límites](../01_Descubrimiento/harmless_real_prefix_entry_0_8_8.md).

## Resultado host del intento consentido

**LOCAL OBSERVED / BLOCKED:** protected path `Steam/userdata/1416423935/config/localconfig.vdf` cambió SHA-256/mtime/inode entre review snapshot70a8779973fde696036eabd53ce9067a y preflight snapshot3207b5b9493c2beaf767c6b02dd98c88. Size59838 permanece igual. Solo1 diferencia protegida; inventario selectivo prefix y prefix identity permanecen iguales. Causa/writer/keys UNKNOWN. Datos exactos/hashes en [descubrimiento](../01_Descubrimiento/harmless_real_prefix_entry_0_8_8.md).

La comprobación está antes del staging de helper marker, before receipt de ejecución y Proton invoke. No real-prefix-entry-* receipts encontrados. No hubo ejecución, ni mutaciones por helper, ni restore real. Se conserva backup original sin actualizar. No se afirma ausencia de cambios Steam: precisamente se detectó uno. STOP aplicado; no0.8.9 ni0.9; versión0.8.7 y sin commit de fase completada. Tauri/Vite cerrados.

Cambios de esta revisión final son documentación únicamente; quality offline anterior285 PASS/11ignored permanece evidencia del código no modificado. git diff --check final PASS. Preflight review EligibleForReview no equivale a resultado del intento: último intento BLOCKED / ProtectedPathChanged.

## Investigación de estabilidad posterior

READ-ONLY HOST:3 samples en10s sin cambios; probe90s detectó1 cambio semántico fuera de rama4162040, con rama ZZZ estable. Dos checks same-UID puntuales no detectaron steam/steamwebhelper; no determinan writer. No rawconfig/valores persistidos ni proceso controlado. El diagnóstico actualizado señala paths/campos protegidos cambiados; guard/retry policy sin cambios. Test sintético evita disclosure de valores/hashes en error. [Resultados y límites](../01_Descubrimiento/estabilidad_localconfig_preflight_0_8_8.md). Sin nuevo Tauri/Proton/helper run; versión0.8.7 y gate BLOCKED.

Validación del diagnóstico actualizado:286 tests PASS/11ignored; fmt/check/clippy -D warnings y git diff --check PASS. Frontend sin cambios en esta investigación, validado en la preparación anterior. La fuente del probe Rust se formateó y compiló manualmente; no se integra a tests normales ni UI.

## Seguimiento del bloqueo

COMPROBADO: se repitieron fmt/check/clippy workspace y cargo test (host tests ignorados), frontend typecheck/lint/format:check/build. Dos probes Linux read-only bounded240s solapados observaron una misma escritura fuera de Software/ZZZ; no identifican la clave histórica ni autor. Diagnóstico backend mantiene protected_path_changed antes de la invocación. No se ejecutó nuevamente Tauri, Proton ni helper. Versión0.8.7, 0.8.8 BLOCKED y fases posteriores no iniciadas.

Última ventana read-only240s finalizó con0 cambios y sin scanner residual. Reejecución offline final:286 PASS/0failed/11ignored. No nueva validación nativa del mensaje ampliado; UI anterior sí validó el bloqueo, render actualizado NO COMPROBADO en Tauri.

## Nuevo intento consentido — resultado definitivo de esta sesión

**HOST TEST / COMPROBADO:**2026-10-02T02:58:30–39Z, sesión699f4768f21d30842653eb404baeb237. Preflight pasó y helper0.6 protocol1 verificó nonce/marker/hash, exit0/1426ms. Identidad y build permanecen iguales;9 protected records sin diferencias; backup Verified. UI nativa confirma `Result: unexpected_mutation` y panel de diferencias. No pérdida del resultado en wiring demostrada.

**BLOCKED:** system.reg/user.reg cambiaron contenido; dosdevices y root metadata/fingerprint cambiaron fuera de política estrecha; userdef.reg solo metadata con contenido idéntico. Resultado UnexpectedMutation, gate cerrado, STOP sin retry/restore. Detalle/hashes en descubrimiento. No se ejecuta0.8.9/0.9; versión0.8.7; no checkpoint de fase completada.

La sesión inicial bloqueada por localconfig y las investigaciones read-only anteriores se conservan como historia. No se atribuye el fallo nuevo a protected files: esos sí permanecieron iguales esta vez. No se da rollback real por verificado ni se actualiza el backup para acomodar cambios.

Quality final después de registrar el HOST TEST:286 PASS/0failed/11ignored, fmt/check/clippy all-targets/all-features -D warnings y frontend typecheck/lint/format:check/build PASS; git diff --check PASS. Helper Windows no modificado. Tauri/Vite iniciados por el agente fueron cerrados tras leer el resultado; no se controla ni termina Steam/Wine/juego. Sin cambios UI: no se aplica rediseño ni policy para forzar gate.

## Diagnóstico posterior y revisión Twintail

READ-ONLY HOST: backup registry bytes == before hashes y current bytes == after hashes de la sesión; diff semántico exacto permitido. Diagnóstico identifica3 Fonts tables (135added/368rewritten cada una),4 ContainerId-only sections,1 NetworkCards empty removal; userdef content unchanged. Fonts rewrites cambian run/host → usr/share/fonts con mismo filename. Esto acota diferencia de entorno/container, no demuestra causalidad ni aceptabilidad.

Twintail stable pinned97d626e76e5eb0bc01361e8788d3a23133c38429, package2.5.1, GPLv3 source consultado sin copiar/ejecutar. ZZMI map usa hook vía3dmloader.exe; no evidencia nueva para elegir Direct Inject. Source diagnosis bounded/hash verified/values redacted añadido con tests offline; example host read-only, sin runtime/helper execution. Original UnexpectedMutation y gate intactos. [Descubrimiento detallado](../01_Descubrimiento/twintail_y_mutaciones_prefix_0_8_8.md).

Quality del diagnóstico de mutaciones:11 tests sintéticos nuevos,297 PASS/11ignored workspace, fmt/check/clippy all-targets/all-features -D warnings y todos los checks frontend PASS. Example diagnóstico READ-ONLY SOURCE completó tras validar receipt SHA y hash de registros; cero fuentes alteradas dentro de verificación de hashes. Sin nueva ejecución Tauri/Wine/Proton/helper. Versión0.8.7 y gate cerrado.

## Comparación del entorno Linux en fixtures — 2026-10-02

**HOST TEST / COMPROBADO:** cinco ejecuciones del helper existente, sin loader, en un prefix nuevo privado. Repetición de contenedor y directo conservan tablas Fonts; cambiar contenedor a directo reproduce exactamente 135 entradas nuevas y 368 rutas modificadas por tabla del intento real; retorno al contenedor invierte el cambio. Cuatro ContainerId cambian también en las repeticiones del mismo entorno. NetworkCards vacío se remueve en repetición inicial; 2 secciones adicionales de esa repetición siguen no clasificadas. No hay una nueva entrada real ni policy para aceptar esas mutaciones.

**BLOCKED:** guard nuevo LinuxEnvironmentUnqualified en dominio y comandos Tauri; nueva review/baseline/consent no habilita el executor directo conocido. Native Tauri AT-SPI confirmó código y razón en UI; no checkbox/Run, Tauri/Vite cerrados. Helper no modificado. El transporte del ensayo usa archivo privado + shim Linux fijo para superar stdin /dev/null de pressure-vessel, sin shell ni helper más potente.

[Investigación y límites](../01_Descubrimiento/entorno_linux_fixture_0_8_8.md), [reporte redactado](evidencias/fixture_environment_0_8_8.json). SHA-256 del reporte: 688d410e401b39d512907d0e3eecd565c0144cffc158f027dc3874c91e64fb2c. Fuentes fijadas al steam-runtime-tools v0.20260805.0/02cb49d93300fcedd01ce896d770e19b74160af7. Protected state selectivo y registros reales permanecieron iguales durante el ensayo, sin garantía whole-tree.

Quality:304 PASS /11ignored; fmt/check/clippy workspace all-targets/all-features -D warnings; frontend typecheck/lint/format:check/build; git diff --check PASS. Version0.8.7, original UnexpectedMutation intacto, no0.8.9/0.9, commit ni push.

## LXMI 0.8.8.1 — Linux Runtime Environment Qualification (2026-10-02)

**COMPROBADO:** se añaden modelos de entorno/calificación/transporte, reglas de mutación por sección/campo, clones independientes verificados, copia privada de Proton, matriz opt-in y panel Advanced con lectura de evidencia tipada/hash verificado. Continúa **0.8.8 BLOCKED / PARTIAL**, versión **0.8.7**. No nueva entry real ni restore.

**HOST TEST:** tres estrategias, dos runs cada una, desde clones con manifest inicial idéntico. Seis helper PASS más seed PASS: protocolo 1, nonce fresco, mismo marker/hash y exit 0. Direct run 1 cambia +135/368 por tabla Fonts; run 2 estable. Ambos wrappers container conservan Fonts en ambos runs. NetworkCards vacío se omite del hive en run 1; Wow6432Node RunServices/RunServicesOnce aparecen vacíos. Cuatro Device ContainerId varían en cada run final. Secciones detalladas UNKNOWN=0, mappings y root identity iguales. La política no inspecciona todos los archivos posteriores.

**UPSTREAM VERIFIED / DERIVED:** pins de Steam Runtime Tools `02cb49d93300fcedd01ce896d770e19b74160af7`, Proton `22f5f694d698d786a9e8ff91d559147c8189a889`, Wine `adea6cbac9d00bb22910145354d00c463fcf3afc`. Fonts/remap, dispositivo PnP/GUID, RunServices vacío y serializer de claves volátiles explican los grupos con reglas concretas. NetworkCards representa omisión del registro guardado, no demostración de eliminación del registro vivo. [Detalle y fuentes fijadas](../01_Descubrimiento/entorno_linux_fixture_0_8_8.md).

Artefacto final: [matriz](evidencias/environment_qualification_0_8_8_1.json), SHA-256 `0797a5ed44ea490ad53b5ac582d80aa77360f57cfe3d25a9ac2e81c15e509403`; [resumen UI](evidencias/environment_qualification_0_8_8_1_summary.json). Copia de Proton SHA manifest `9d0bb73e241bac979afeda526b589fce6f26ee2d5e8822b7aadc710b179421bb`; clones iniciales `9ad5ac3573494d72f5ce9997a336ba4c1206eded9961731fdd110089e2341eaa`. Source sin cambios durante copy completa; metadata selectiva del tool instalado sin cambios en las seis ejecuciones. No ejecución del tool instalado en la matriz final.

Matrices anteriores preservadas: [tool instalado](evidencias/environment_qualification_0_8_8_1_installed_tool.json), [revisión limitada de font paths](evidencias/environment_qualification_0_8_8_1_installed_tool_font_review.json). No se califican para la frontera de escritura: Proton puede truncar `dist.lock` en el tool instalado. Un seed inicial dentro del sandbox también devolvió malformed response; no se contó como éxito. La matriz final se ejecutó con aprobación nativa, source mirror y bytecode Python desactivado; no hubo fallback ni reintento dentro de un prefix real.

Tests offline cubren calificación, clones/mounts/pins/metadata/env inválidos, Unknown que no desaparece entre VolatileExpected, ContainerId estricto, NetworkCards no vacío, RunServices no vacío, TeX y separadores dobles, request mode/symlink/stale/hash/schema/nonce/cleanup, transporte explícito, mirror requerido, no secrets heredados y guard real. Los tests host nuevos permanecen ignorados por defecto; el ejemplo opt-in fue usado explícitamente para la matriz privada. El helper Windows no cambió; SHA permanece `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2`.

**QUALIFIED:** recetas container privadas, build/payload/cobertura fijados. **BLOCKED:** executor real DirectProton todavía unqualified, sin implementación real-container aprobada. Calificación de fixture no equivale a entry real permitida. Backend mantiene `linux_runtime_environment_unqualified`; no frontend override. No Proton/Wine/helper contra prefix real, juego, loader, Steam control, cambios de configuración ni restauración real en este subincremento.

### Validación final 0.8.8.1

**COMPROBADO:** `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`: **325 PASS / 0 failed / 12 ignored**. Tests normales offline no ejecutan Proton ni inspeccionan Steam/procfs real. Frontend `npm run typecheck`, `lint`, `format:check`, `build` y `git diff --check`: PASS. El helper Windows no fue modificado ni reconstruido en este subincremento.

**COMPROBADO — Tauri nativo:** `npm run tauri:dev` abrió LXMI 0.8.7. Validación read-only mediante AT-SPI: Scan Steam, Advanced Prefix Preflight y Review; se verificaron dos recetas privadas `steam_runtime_qualified`, SHA del artefacto final, executor actual `unqualified`, el motivo concreto de falta de integración real-container y botón Run deshabilitado. No consentimiento ni Run, no browser-only. Tauri/Vite propios cerrados después; no control de Steam/Wine/juego. Recibo mínimo en `environment_qualification_0_8_8_1_native.json`; no se guarda el texto completo de otras aplicaciones.

**BLOCKED:** 0.8.8 sigue parcial, versión 0.8.7. La mutación real original sigue preservada, sin retry ni restore. El subincremento califica recetas privadas y mantiene separado el executor real todavía no calificado. Sin commit/push ni inicio de 0.8.9/0.9.

## LXMI 0.8.8.2 — integración calificada, gate real detenido (2026-10-02)

**COMPROBADO:** receta tipada compartida entre fixture y entry real, fingerprint/pins, copia completa Proton verificada, wrapper `run`, transporte privado y binding nonce/hash/identidad. Capacidad de executor solo después de self-test privado en la sesión; no se acepta Qualified del frontend ni fallback DirectProton. Native transport bridge-only agregado; helper Windows sin modificaciones.

**READ-ONLY HOST / BLOCKED:** backup original Verified + restore Verified off-target, pero `recovery_source_current_mismatch`. Preflight actual NeedsEvidence con baseline_not_captured/rollback_insufficient_metadata_only; UID 1000, device 66306, inode 11538172, SameBuild, 0 actividad/0 unresolved. No consentimiento nuevo, no entry real, no nuevo baseline/backup/restore para hacer pasar el gate. No cambios atribuidos a LXMI sobre Steam/juego/prefix durante este subincremento; la inspección selectiva no acredita todo el árbol.

**NO COMPROBADO:** helper real con esta receta, before/after de una nueva ejecución real, helper junto a ZZZ activo, ZZMI/ZZZ y loader. La versión permanece **0.8.7**; 0.8.8 no cierra, 0.8.9/0.9 no iniciados. [Diseño, límites y evidencia](../01_Descubrimiento/entorno_linux_fixture_0_8_8.md#lxmi-0882--integración-del-executor-calificado-2026-10-02).

### Executor integrado: HOST TEST final

Seed y dos ejecuciones medidas del mismo código compartido: PASS / **Qualified**. Fonts estables, cuatro ContainerId VolatileExpected, NetworkCards vacío y dos RunServices vacíos StableExpected, UNKNOWN=0; metadata/mappings/root identity y copia completa de Proton verificados. Tres recibos de pressure-vessel con mounts RO, request/context cleanup. [Artefacto final](evidencias/executor_qualification_0_8_8_2.json), SHA-256 `35aaf59e5cf992faf7ed67d94423c3ef043adf463258c7c4d7215f6c116c0dd8`; [recibos container](evidencias/executor_container_receipts_0_8_8_2.json). No se contó la corrida preliminar como verificación del código final.

Quality: fmt --check, check workspace, Clippy workspace all-targets/all-features -D warnings, cargo test workspace: **335 PASS / 0 failed / 12 ignored**. Frontend typecheck/lint/format:check/build PASS; native transport build PASS; git diff --check PASS. Helper Windows no modificado, sin regresión de payload ni rebuild innecesario. Tests normales permanecen offline, sin Steam real/Proton.

**COMPROBADO — Tauri nativo:** Scan Steam, calificación privada final, consulta read-only del prefix y lectura de resultados. Executor Qualified / 2 measured runs, private Proton, runtime/wrapper/pressure-vessel/hash visibles. Preflight NeedsEvidence con los tres blockers concretos; Review y Run disabled/sensitive=false verificados por AT-SPI. Historia DirectProton preservada. Sin checkbox/consentimiento ni Run reales; sin browser-only. [Recibo nativo mínimo](evidencias/executor_qualification_0_8_8_2_native.json). Tauri/Vite propios cerrados después; no control de Steam/Wine/juego. No commit/push ni bump de fase completada.

## LXMI 0.8.8.3 — Recovery Source Reconciliation (2026-10-02)

**READ-ONLY HOST / COMPROBADO:** viejo backup f3551122468ed68b825a0182922f3be2 re-verificado:2445 matching entries/783 hashes, manifest SHA db1d884b02ede5435f64520b92fcddf9e4688d56701c1329536d9abe38fa155f; restore existente Verified off-target. UID1000/dev66306/inode11538172, SameBuild, actividad0/unresolved0. Fuente Proton completa coincide con manifest calificado. Fuente prefix consistente durante comparación.

Diff:0 added/removed/type/symlink-target changes;2 modified (system.reg/user.reg);35 metadata-only (root, dosdevices,32 COM symlinks,userdef.reg). Contenidos actuales coinciden exactamente con after histórico DirectProton, backup con before. Fonts históricos +135/368 por tabla permanecen; cuatro GUID narrow VolatileExpected; NetworkCards vacío omisión de serialización; RunServices/Once sin diferencias en source real. Ningún material prefix UNKNOWN. Artefacto completo SHA 7b286dea15af063741ff4b6c5235838ac0dfe2bbda94e4f8fbdfa803c646a098.

**BLOCKED — ProtectedStateChanged:** config.vdf y localconfig de1416423935 difieren de0.8.7. Sus hashes actuales ya eran iguales antes/después del intento DirectProton, pero claves/autor/semántica no se pueden probar desde baselines hash-only. No normalización/atribución automática ni nueva generación de recovery. Preflight NeedsEvidence; versión0.8.7. No baseline/backup/restore nuevo, consentimiento, helper ni Proton real; no0.8.9/0.9.

**COMPROBADO — Native Tauri:** reconciliación/diff/assessment/historia visibles; fresh baseline/new backup/Review/Run disabled. Recibo mínimo; Tauri/Vite propios cerrados. Backend no acepta assessment frontend; refresh exige source fingerprint, contexto, build, protected state y old backup revalidados. Tests synthetic full recovery generan IDs nuevos y conservan backup/baseline anteriores; no acceso real en tests normales.

[Descubrimiento detallado](../01_Descubrimiento/reconciliacion_recuperacion_prefix_0_8_8_3.md), [diff completo](evidencias/recovery_source_drift_0_8_8_3.json), [correlación histórica](evidencias/recovery_history_correlation_0_8_8_3.json), [recibo nativo](evidencias/recovery_reconciliation_0_8_8_3_native.json). Incremento0.8.8 PARTIAL, sin bump ni fase de loader.

### Quality final 0.8.8.3

**COMPROBADO:** `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` y `cargo test --workspace`: PASS, **340 tests passed / 0 failed / 12 ignored**. Frontend `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`: PASS. `git diff --check`: PASS. Helper Windows sin modificaciones; no reconstrucción ni ejecución necesaria para este diagnóstico read-only. Validación manual Tauri nativa completada con controles de refresh/entry bloqueados; sin sustitución browser-only.

## LXMI 0.8.8.4 — Steam Protected State Qualification (2026-10-02 local / 03 UTC)

**READ-ONLY HOST / REVIEWED / STABLE:** dos VDF fijos, parser KeyValues existente y decoder binario acotado del mismo AST. CompatToolMapping presente en config.vdf, mappings 4162040/global 0 ausentes. Launch options/command overrides ZZZ ausentes. Localconfig UserAppConfig/4162040 decodifica únicamente internal/language=english. Secrets, depots/cloud/tickets y estado privado excluidos. Dos lecturas separadas por 5 segundos: raw hashes/metadata/fingerprint iguales. Fingerprint relevante `8677e768e0fc2a42985ea4c67a3cc20bc52b38900bb0a9affbfac6832cde0d70`. Writer **UNKNOWN**, diff histórico exacto **NOT RECONSTRUCTABLE**; no atribución a DirectProton.

**COMPROBADO:** SteamProtectedStateAssessment ReviewedAndAcceptable; CurrentPrefixStateAssessment UnderstoodAndAcceptable. Los dos raw changes históricos quedan CurrentSteamStateReviewed, sin inventar sus claves antiguas. Política mantiene raw hashes diagnósticos y gate semántico vinculado al snapshot, con TTL 600 s; otros paths estrictos, Unknown/overrides/mappings no aprobados fail closed. Lectura no sigue symlinks, bounded 2 MiB, owner/stamps y AppID fijo. Journals de drift ya no se confunden con journals de backup.

**HOST TEST / VERIFIED:** baseline schema 2 nuevo `zzz-prefix-574e7c92055600c88f9d2484d2857e55.json`; backup actual `f4ea33c0c047d893319296420fd86784`, manifest SHA `182929afbefb7e08b226cb7e1651a44a66df95e55e12422c58752bf81ed40f80`. 2445 matching entries/783 file hashes; fuente consistente y protected paths sin cambios observados. **VERIFIED OFF-TARGET:** restore nuevo en LXMI sandbox, sin mismatch/unsupported metadata. Backup histórico f3551122468ed68b825a0182922f3be2 preservado/verificado. Real restore **NOT TESTED**, sin executor.

**COMPROBADO — Native Tauri:** Scan Steam, review semántica, assessment/diff, baseline nuevo, backup/verify/restore, reevaluate; ambos backups visibles. Una revisión expiró durante espera externa y fue rechazada por el gate; se renovaron solo lectura/baseline y reevaluación, conservando TTL y backup. Resultado final puntual **ELIGIBLE FOR REVIEW**, UID1000/dev66306/inode11538172, SameBuild, contexto inactivo. Tauri/Vite propios cerrados. Sin checkbox/consent, self-test Proton ni helper real; no browser-only.

Quality final: fmt --check, check workspace, Clippy workspace all-targets/all-features -D warnings PASS; cargo test workspace **353 PASS / 0 failed / 12 ignored**. Frontend typecheck/lint/format:check/build PASS; git diff --check PASS. Tests cubren extracción/mappings, launch/redaction, binario/duplicados/truncamiento, estabilidad raw-vs-semantic, unknown/parsing/symlink/size fail closed, asociación raw/fingerprint, protected strict, review requerida y journal selector. Helper sin modificaciones. Normal tests sin Steam real/Proton.

[Investigación y decisión](../01_Descubrimiento/estado_protegido_steam_0_8_8_4.md), [semántica seleccionada](evidencias/steam_protected_state_0_8_8_4.json), [reconciliación](evidencias/recovery_source_reconciled_0_8_8_4.json), [baseline](evidencias/prefix_baseline_current_0_8_8_4.json), [recovery](evidencias/prefix_recovery_current_0_8_8_4.json), [validación nativa](evidencias/steam_protected_state_0_8_8_4_native.json).

**STOP:** 0.8.8.4 completa; versión **0.8.7**, 0.8.8 aún abierto. Próxima entry requiere calificación de executor en su sesión, checks frescos y consentimiento separado. No helper/Proton/Wine/ZZZ/loader/restore real en este subincremento; 0.8.9/0.9 no iniciados. Sin commit/push; worktree previo conservado.

## LXMI 0.8.8 FINAL — preparación en sesión nueva (2026-10-03)

Checkpoint inicial `ec898fe`, main, worktree limpio, versión0.8.7. Tauri nativo abierto; self-test integrado recalificado en la sesión privada `aa90ab7eb6b14abb3d0b49b683623efa`. Seed y dos runs medidos; estos últimos helper/nonce/marker/hash PASS, exit0, fonts estables, primer run3 StableExpected+4 VolatileExpected, segundo4 VolatileExpected. **QUALIFIED**, recipe fingerprint `6ff2ca659cd2db24d246c4cfa321fba1788203e4c956cebddbe161d50c841aed`, copia Proton y runtime/pressure-vessel/wrapper/env/transporte verificados, sin fallback directo. [Artefacto completo privado](evidencias/executor_qualification_0_8_8_final.json), SHA-256 `32ea23922508809591d50d453901407c4bdcc16ba82d608459182c50180ba3b6`.

Revisión Steam read-only nueva **ReviewedAndAcceptable / Stable**, source **UnderstoodAndAcceptable**. Backup actual f4ea33c0c047d893319296420fd86784 y restore off-target revalidados; contexto inactivo, SameBuild e identidad UID1000/dev66306/inode11538172 iguales. Baseline de review nuevo `zzz-prefix-446154f834857d32df595e85dd8404d2`, preservando anteriores; [snapshot](evidencias/prefix_baseline_0_8_8_final_review.json), SHA-256 `550fecb988c56d5ec3545fe2d36cefe14382c6ad0b443500aba856034c694cb0`. [Steam review](evidencias/steam_protected_state_0_8_8_final_before.json), SHA-256 `5a36df10b5b8a44aa51f2b080c884da67df1b9970d758e15653c123c5f27ab97`.

**COMPROBADO — Tauri nativo:** Scan Steam, self-test privado, review Steam, baseline, reevaluación EligibleForReview y Review Harmless Prefix Entry preparado. [Recibo nativo previo al consentimiento](evidencias/qualified_real_entry_0_8_8_final_review.json). Casilla de consentimiento sin marcar y Run deshabilitado al terminar esa preparación; sin reserva one-shot ni ejecución real. Se solicitó al usuario consentimiento nuevo mediante la interfaz nativa. No se reutiliza el anterior ni se marca por el agente. Pruebas Rust **353 PASS / 0 failed / 12 ignored**, fmt/check/Clippy PASS; frontend typecheck/lint/format:check/build y git diff --check PASS.

**PENDIENTE:** consentimiento nuevo, UNA ejecución real y su before/after. Esta sección documenta preparación, no resultado de entry. La prueba real conserva TTL/gates; si un guard falla se detiene sin retry ni restore. Versión0.8.7, sin commit de cierre, 0.8.9/0.9 no iniciados. No ZZZ, loader, Hook, Inject ni real restore.

## LXMI 0.8.8 FINAL — resultado del único intento (2026-10-03)

### Session executor qualification

**QUALIFIED / COMPROBADO:** receta `6ff2ca659cd2db24d246c4cfa321fba1788203e4c956cebddbe161d50c841aed`, Steam Runtime4/pressure-vessel/wrapper run, copia Proton privada/build aprobado, transport/env allowlist; seed+2 runs privados correctos en esta sesión. Calificación privada no se convierte en aceptación de una mutación real.

### Fresh preflight

**VERIFIED:** revisión fresca EligibleForReview, Steam actual ReviewedAndAcceptable/Stable5s, source UnderstoodAndAcceptable, backup f4ea33c0c047d893319296420fd86784 y restore off-target revalidados. Identidad UID1000/dev66306/inode11538172, SameBuild, game/context inactivos. Baseline de review anterior y baseline inmediato `zzz-prefix-12dd5c0db00556232f5fc872be7b2fb6` conservados; el estado before real está en el recibo del intento.

### Consent

**COMPROBADO:** consentimiento NUEVO aceptado por el usuario en Tauri; backend registra consent_accepted=true. No se reutilizó el consentimiento DirectProton ni se marcó la casilla por el agente. El usuario confirmó que pulsó Run; reserva one-shot persistente correspondiente a nonce `213181058a3f24c980a390dd5ab2797b`.

### Real helper execution

**COMPROBADO:** UNA invocación real mediante receta calificada. Sin retry, otra autorización ni restore automático. Reserva conservada; backend rechaza otro prepare/run para esta prueba. [Before](evidencias/qualified_real_entry_0_8_8_final_before.json), SHA `a51836efa06809ab32dd76ec4d3eeb320b2b7dba408c5c2bc21fcef3b46d72a2`; [after](evidencias/qualified_real_entry_0_8_8_final_after.json), SHA `5613b8dbf08792f02318585f299fb6d08718e74fe13ed052b67e8a3534027902`.

### Helper protocol

**VERIFIED:** protocolo1, handshake succeeded, nonce igual al de la sesión, marker externo visible por Z:, hash y helper SHA correctos, exit0, 2048ms del proceso medido. Context/request/runtime receipt temporales ausentes tras cleanup. [Container receipt](evidencias/qualified_real_entry_0_8_8_final_container.json), SHA `ce37827b9f7af21a16ac03cfdf38b08ede994db03ab09ed0d4e11e8d9d254073`; [resumen de checks](evidencias/qualified_real_entry_0_8_8_final_summary.json). No DLL/loader/ZZMI.

### Before / after

12 entradas diagnósticas: root y dosdevices Unknown bajo el clasificador actual; system.reg/user.reg PossibleWineMutation; userdef.reg metadata con contenido igual ExpectedWineMutation; tres tablas Fonts EnvironmentSpecific representadas como Unknown bloqueante; cuatro GUID ContainerId VolatileExpected representados como ExpectedWineMutation. No nuevas reglas de aceptación.

El diagnóstico de inventarios observa los mismos nombres y targets de symlinks; cuatro entradas del root cambian metadata y 32 mappings COM cambian inode/mtime en dosdevices. Se mantienen como evidencia sin ampliar ignores. Metadata root/device/inode no cambió; timestamps internos y sustitución de hives sí. No se afirma cobertura de todo el árbol ni ausencia universal de otras mutaciones.

### Font state

**BLOCKED / ENVIRONMENT SPECIFIC:** tres secciones cambiaron: system.reg Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts; system.reg Software\\Microsoft\\Windows\\CurrentVersion\\Fonts; user.reg Software\\Wine\\Fonts\\External Fonts. El self-test privado inició desde estado container, mientras el source real aceptado conservaba estado DirectProton histórico.

**COMPROBADO:** el hash de la tabla NT before real coincide exactamente con after del fixture DirectProton run1, y after real con before container de ese fixture. Es una reversión de esa sección concreta hacia el estado container conocido. En las otras dos tablas no existe esa igualdad exacta; la relación con la transición de entorno queda **DERIVED**, sin inventar counts reales ni causalidad total. El fixture histórico cambia503 fields por tabla; eso no se presenta como conteo medido de las otras tablas reales. [Correlación acotada](evidencias/qualified_real_entry_0_8_8_final_font_correlation.json).

### ContainerId

Cuatro cambios **VOLATILE / EXPECTED** cumplen reglas GUID estrechas existentes. No se aceptaron otros campos por wildcard.

### NetworkCards

Sin diferencias reportadas por la comparación calificada de hives. No nueva eliminación ni regla añadida; esto no inspecciona registry vivo mediante APIs Windows.

### Additional registry state

RunServices/RunServicesOnce: sin diferencias reportadas. userdef.reg conserva contenido hash, cambia metadata. No cambios registry desconocidos fuera de los grupos detallados reportados por el comparador.

### Steam protected state

**VERIFIED:** semántica relevante y raw evidence protegida antes/después iguales. Sin nuevas launch options/overrides/mappings. Fingerprint `8677e768e0fc2a42985ea4c67a3cc20bc52b38900bb0a9affbfac6832cde0d70`. Se conserva la diferencia raw histórica y writer UNKNOWN.

### Protected paths

**VERIFIED dentro de cobertura:** ejecutable ZZZ, metadata seleccionada de instalación, appmanifest, Steam config/localconfig, packages y managed runtimes sin cambios en los snapshots protegidos. No claim whole-game/whole-disk. Manifests históricos y actual de backup conservan sus SHA originales; backup Verified después. Representa source PRE-ENTRY, no se actualiza para simular equality con source post-test.

### Prefix identity

**VERIFIED:** canonical pfx, UID1000, device66306, inode11538172 iguales. Runtime SameBuild. game_became_active=false, contexto inactivo al check final; observación puntual/periódica no es lock atómico ni garantía universal de proceso.

### RealPrefixEntryResult

**BLOCKED — UnexpectedMutation.** Helper PASS no basta: font-state no permaneció estable y sus cambios no satisfacen las categorías permitidas por la política real. No se promociona a VerifiedWithQualifiedVolatileMutations, no se normaliza registry, no retry ni restore real. 0.8.8 no cierra.

### Native Tauri validation

**COMPROBADO:** el usuario realizó consentimiento/Run en Tauri nativo. El agente leyó el resultado y guardó evidencia con AT-SPI propio. Se corrige el texto fijo Not run y loading de intento; header obtiene versión del package0.8.7. El lector de historial nuevo, zero-arg, lee solo dos artefactos fijos LXMI por descriptores y reconstruye recibo SHA; no prefix/Steam ni executor. Reserva presente mantiene attempted=true, resultado ausente no habilita ejecución, lectura insegura mantiene controles bloqueados. Basado en archivo/reserva, no frontend override ni consentimiento recuperado.

Validación tras recarga: resultado UnexpectedMutation/Helper Success/paths/backup/identidad visibles, Run y Review disabled; [recibo nativo](evidencias/qualified_real_entry_0_8_8_final_native.json). La etiqueta del header no fue expuesta en esa vista AT-SPI; versión confirmada por package/build y corrección de fuente, sin claim de inspección visual de ese nodo. Tauri queda abierto para revisar el resultado; no nueva calificación, review ni ejecución host después del bloqueo. La historia se muestra separada de la calificación de la nueva sesión.

### Tests

Quality FINAL: fmt --check, check workspace, Clippy workspace all-targets/all-features -D warnings PASS; cargo test workspace **358 PASS / 0 failed / 12 ignored**. Frontend typecheck/lint/format:check/build PASS; git diff --check PASS. Cinco tests nuevos de historia: ausencia no crea archivos, reserva sin resultado sigue consumida, recuperación de resultado/binding nonce/receipt sin capability, traversal/symlink, limits/modes y directorio legacy bajo root privado. Normales sintéticos sin Steam/Proton reales. Helper Windows sin modificaciones ni reconstrucción.

### Version

**0.8.7**, sin bump. El header0.8.6 era literal desfasado; no era la versión package/Tauri. Ahora procede del package.

### Git state

Base main `ec898fe`; evidencia/docs y corrección de observabilidad sin commit de cierre ni push. No se etiqueta funcionalidad0.8.8 como completada.

### Remaining blockers

La receta está calificada para el fixture medido, pero no pasó el gate de transición del source real con fonts DirectProton históricos. Para futura investigación se requiere una topología privada que represente ese estado inicial y estudie transición/repetición e impacto; no se ejecuta ahora ni se cambia aceptación para obtener PASS. Source real post-test difiere del recovery pre-entry; backup conserva integridad y valor de recuperación, no igualdad actual. 0.8.9/0.9 no iniciados. Real restore NOT TESTED. ZZMI/ZZZ y loader permanecen UNKNOWN.

## Continuación privada — transición desde el backup pre-entry (2026-10-03)

**HOST TEST:** dos clones completos independientes del backup `f4ea33c0c047d893319296420fd86784`, dos runs por clone. Cuatro helpers PASS; estado inicial completo idéntico, backup Verified/unchanged. En run 1: 135 remociones/368 rutas modificadas por tabla; run 2: fuentes estables, cuatro ContainerId bajo regla existente. Hashes antes/después de las tres tablas coinciden exactamente con las tres secciones del intento real fallido en ambas réplicas. NetworkCards/RunServices/RunServicesOnce sin cambios, metadata seleccionada y root identity privadas correctas. Sin nuevas ejecuciones reales.

**COMPROBADO:** añadido guard backend `initial_prefix_state_unqualified`, fingerprints de tres tablas medidas en self-test, check antes de emitir Review/token y tras final preflight antes de Proton real. Sin ampliar las reglas de mutación. El guard nuevo se validó con tests sintéticos; el branch real no se ejecutó. Receta de diagnóstico privada anterior al añadido del guard preservada como evidencia separada.

fmt/check/clippy, **363 tests PASS / 12 ignored**, frontend typecheck/lint/format/build y diff check PASS. Native Tauri confirmó historial del fallo y Review/Run deshabilitados después de reload. Versión 0.8.7, 0.8.8 BLOCKED, sin commit/push/0.8.9/0.9.

[Investigación, alcance y próximos requisitos](../01_Descubrimiento/transicion_fuentes_backup_privado_0_8_8.md). Evidencia: `private_font_transition_0_8_8.json` y summary/native en `evidencias/`. La reserva real y ambos backups históricos permanecen preservados.

## Corrección del estado inicial y recuperación actual — 2026-10-03

**COMPROBADO:** calificación del executor vinculada a generación actual verificada. `qualify_integrated_executor_from_recovery` utiliza el motor de backup/clone existente y la misma receta; verifica backup y restore, mide el primer helper desde los bytes originales del clone y después otro run. No pre-normalización ni DirectProton fallback. Review/backend requieren recovery ID/hash y tres fingerprints iniciales; generic qualification no autoriza real entry. El guard one-shot histórico sigue activo.

**READ-ONLY SOURCE:** revisión fresca Steam ReviewedAndAcceptable/Stable, fingerprint relevante sin cambios. Reconciliación UnderstoodAndAcceptable, cero material UNKNOWN; identidad real UID1000/dev66306/inode11538172, SameBuild/inactividad. No nuevos cambios semánticos de fuentes en la comparación actual frente al backup original. La aceptación del source actual no reclasifica retroactivamente el resultado real.

**VERIFIED / VERIFIED OFF-TARGET:** nueva generación `2b29b84bbc592fa091950bf046b3d668`, manifest `79685d4cdf3523d6acd5ab435a33a175438f488b871d13be3dd6cae0b8770597`, 2445 entradas/783 archivos; integrity, source consistency, selected protected paths y sandbox restore PASS. Ambas generaciones anteriores preservadas. Real-prefix restore NOT TESTED. Nuevo baseline final `zzz-prefix-64dac842206897b341916bed5974b11e`, SHA `23bef28c6190fd6d0b5e163b4b476820f0b0aff9950922cd63b1bc39049e5c36`.

**HOST TEST / Native Tauri:** nueva receta source-bound Qualified, dos runs privados en sesión `ed785c1a68f15695671b1d4130a166b2`; protocol1/nonce/hash/exit0 PASS (2737/2029ms), fonts estables desde run1, cuatro GUID ContainerId VolatileExpected por run, cero UNKNOWN. NetworkCards/RunServices/RunServicesOnce sin diferencias semánticas; metadata seleccionada, request cleanup y private Proton integrity correctos. Source actual y protección read-only revalidados después. Preflight final EligibleForReview. UI muestra recovery seed actual/Qualified e historia; Review/Run reales disabled, no consentimiento/ejecución real.

**Tests:** fmt --check, check --workspace, Clippy all-targets/all-features -D warnings; cargo test --workspace **365 PASS / 0 failed / 12 ignored**. Typecheck/lint/format:check/build frontend y git diff --check PASS. Tests nuevos cubren recovery binding ausente/id distinto/hash distinto/Unqualified; aceptación/UNKNOWN/protected/activity/identity para refresh, reuse si source coincide. Integración sintética ahora verifica journals/manifests de ambas generaciones preservadas. Helper Windows sin cambios.

Artefactos: [summary](evidencias/current_source_requalification_0_8_8_summary.json), [executor](evidencias/current_source_executor_qualification_0_8_8.json), [native recovery](evidencias/current_source_recovery_0_8_8_native.json), [native qualification](evidencias/current_source_executor_0_8_8_native.json), [native readiness](evidencias/current_source_readiness_0_8_8_native.json), [baseline seleccionado](evidencias/current_source_baseline_0_8_8.json). Sin full registry/Steam config dumps. Recipe report describe solo su ejecución privada; el servicio controller sí lee el source real para revalidarlo.

**Pendiente:** versión0.8.7, sin commit de cierre/push. El fallo real sigue UnexpectedMutation y one-shot consumido; hashes de reserva/recibo y manifests anteriores intactos. Los problemas de calificación/recuperación tienen evidencia nueva positiva, pero una nueva entrada real necesita autorización explícita para otro experimento y luego consentimiento nativo, por la regla FINAL de un solo intento/detener ante fallo. No implementación de reintento, no real restore,0.8.9/0.9.
