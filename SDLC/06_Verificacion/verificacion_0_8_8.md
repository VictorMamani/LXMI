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
