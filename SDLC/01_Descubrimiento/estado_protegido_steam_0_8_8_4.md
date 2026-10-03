# LXMI 0.8.8.4 — Steam Protected State Qualification

Versión 0.8.7. Este subincremento revisa Steam y renueva recuperación; no cierra 0.8.8. No consentimiento, helper, Proton/Wine, juego, loader ni restauración real.

## Decisión y alcance

Los hashes raw históricos distintos permanecen como diagnóstico. No es posible reconstruir el diff histórico exacto de claves a partir de los baselines hash-only de 0.8.7; autor **UNKNOWN**. Se acepta únicamente el estado semántico ACTUAL, extraído y observado estable por el backend. No se atribuye la escritura a DirectProton ni a un proceso concreto de Steam.

Dos archivos se revisan semánticamente: `config/config.vdf` y `userdata/1416423935/config/localconfig.vdf`, derivados del Steam root canónico. Continúa la protección estricta del resto del conjunto selectivo existente. No hay inspección de documentos, amigos, chats, actividad privada o dumps completos.

## Parser y privacidad

Se reutiliza `lxmi-steam::parse_key_values_document`, con límite existente de 2 MiB y nesting 64. Lectura por descriptores `openat`, padres/directorio/fichero sin seguir symlinks, owner actual, tipo regular, `O_NOATIME`, tamaño y stamps antes/después. Se rechazan rutas ajenas, directives, duplicados relevantes, campos relevantes desconocidos y escrituras durante una lectura.

El host contiene `UserAppConfig/4162040` como scalar hexadecimal, no como objeto de texto. Se amplía el módulo KeyValues existente con un subset binario acotado: objeto tipo 0, string tipo 1, terminador tipo 8; máximo 8192 caracteres hex, profundidad 16, 64 entradas por nivel. Tipos adicionales, truncamiento, trailing bytes o root/campos inesperados bloquean. Reutiliza el mismo AST tipado; sin regex ni lector paralelo de config textual.

**UPSTREAM VERIFIED:** esos tipos y el formato recursivo/string terminado en NUL aparecen en Valve Source SDK 2013, commit `b8cfb12c0e083a2ef5b2f9f9b50f3902fa034474`: [types_t y KeyValues.h](https://github.com/ValveSoftware/source-sdk-2013/blob/b8cfb12c0e083a2ef5b2f9f9b50f3902fa034474/src/public/tier1/KeyValues.h), [ReadAsBinary](https://github.com/ValveSoftware/source-sdk-2013/blob/b8cfb12c0e083a2ef5b2f9f9b50f3902fa034474/src/tier1/KeyValues.cpp). Consultado 2026-10-02/03 UTC. Esto verifica el formato; no documenta todas las semánticas privadas del cliente Steam. **COMPROBADO** en este host: el scalar decodifica a `internal/language = english` y no tiene otros campos.

Steam documenta que las opciones de lanzamiento pueden sustituir ajustes del juego: [Steam Support](https://help.steampowered.com/en/faqs/view/7D01-D2DD-D75E-2955). Por eso cualquier launch option no vacía/override desconocido bloquea; LXMI no ejecuta ni modifica esa configuración.

Exclusiones explícitas: tickets y autenticación, decryption keys/depots, cloud, friends/chat, shader cache/storage metadata, herramientas/size on disk, badges, EULA y usage/recent-play. No se persisten sus contenidos ni hashes de sus valores individuales. La extracción recorre el documento en memoria y retiene solamente campos relevantes/permitidos; referencias al juego fuera del schema reconocido quedan redacted/Unknown. Los valores de comandos se redactan y se conserva su huella, sin mostrarlos ni ejecutarlos. Las literals permitidas son limitadas a lenguaje, nombres runtime reconocibles y campos numéricos seleccionados.

## Estado ACTUAL — READ-ONLY HOST

| Evidencia | config.vdf | localconfig.vdf |
|---|---|---|
| Hash raw | `b69ced69a04f3a2fe810a5428642941e53ccd9f47b8e1347f7a0195eaa3bd1c7` | `a3570c17697303b91924405d72361f52f0110ec2eda8b8328e0b8717b0da68f4` |
| Raw contra 0.8.7 | Changed; diferencia histórica preservada | Changed; diferencia histórica preservada |
| CompatToolMapping | Present | Absent en este documento |
| Mapping 4162040 | Absent | Absent |
| Mapping global 0 | Absent | Absent |
| Launch options / overrides ZZZ | Absent | Absent |
| Settings Steam Play explícitos seleccionados | No encontrados | No encontrados |
| Otras propiedades relevantes seleccionadas | Ninguna | `internal/language = english` |

La selección configurada sigue **Unknown**, separada del Proton Experimental **Observed** previamente y del runtime aprobado por build. No se reescribe la evidencia histórica. Mapping nuevo no significa inseguridad automática, pero exige revisión/aprobación adicional; no se aprueba silenciosamente.

Observación read-only: dos lecturas completas/selección antes-después, intervalo acotado de 5 segundos. **STABLE**: hashes, metadata y fingerprints seleccionados iguales. Fingerprint semántico conjunto `8677e768e0fc2a42985ea4c67a3cc20bc52b38900bb0a9affbfac6832cde0d70`. No se confunde esto con equivalencia semántica respecto de 0.8.7: ese contenido anterior no está disponible.

## Modelos y gates

`SteamProtectedSemanticState` conserva schema/policy, timestamp, AppID, raw evidence de ambos archivos, mappings separados, opciones/overrides redactados, propiedades seleccionadas, categorías excluidas, issues y fingerprints. `SteamProtectedStateReview` conserva ambas observaciones, intervalo, stability, assessment, límites y autor histórico.

Resultados distinguen `Stable`, `RawChangedSemanticStable`, `SemanticChanged`, `Unknown`. Un raw hash nuevo con fingerprint relevante idéntico puede ser `UnrelatedSteamState` **solo entre dos snapshots semánticos válidos y vinculados a sus propios hashes/metadata**. No es una explicación de los bytes históricos nunca almacenados.

Assessment aceptable requiere estado relevante entendido, sin overrides, sin mappings nuevos sin aprobación, sin campos desconocidos, y observación estable. La prueba tiene TTL backend de 600 segundos y se revalida antes de baseline/copia. No existe parámetro frontend para aprobar assessment.

Un baseline nuevo schema 2 incluye raw hashes más snapshot semántico; los schema 1 históricos siguen legibles mediante campo opcional y no adquieren una exención semántica. Paths protegidos restantes mantienen comparación estricta. Baseline, recovery, restore y actividad/build/identity se verifican nuevamente; source fingerprint completo debe coincidir con el reconciliado. El backup antiguo y su manifest inmutable se preservan. El executor sigue exigiendo calificación privada de la sesión y consentimiento independiente antes de una futura entrada; esta revisión no concede ninguno.

**REVIEWED:** `SteamProtectedStateAssessment::ReviewedAndAcceptable` permite `CurrentPrefixStateAssessment::UnderstoodAndAcceptable`, manteniendo las dos diferencias raw históricas con clasificación `CurrentSteamStateReviewed`, sin asignarles autor ni afirmar que sus claves pasadas eran iguales.

## Límites

La observación no es un lock de Steam ni una garantía de estabilidad futura. Se protege un schema relevante conocido, no todo el estado privado del cliente. Una configuración nueva/ambigua falla cerrada. La recuperación cubre el árbol pfx y metadata soportada, no Steam completo ni instalación del juego. Restore real **NOT TESTED**. El estado de fuentes histórico no se normalizó. El helper real con el entorno calificado sigue **NO COMPROBADO**.

## Recuperación ACTUAL — HOST TEST

La revisión permite aceptar el source actual sin modificarlo. Identidad root UID 1000, device 66306, inode 11538172; runtime SameBuild, contexto inactivo, sin candidatos unresolved. Se conservan las diferencias históricas de system.reg/user.reg y los 35 cambios metadata-only del análisis 0.8.8.3; no hay normalización de fonts ni ejecución de Wine.

Baseline nuevo schema 2: `zzz-prefix-574e7c92055600c88f9d2484d2857e55.json`, SHA-256 `afa1bc51daf4d767f31675f9d82d38de8808d2a1392e14d8ddd239d3c8703c05`. Conserva raw evidence y fingerprint semántico. Los baselines anteriores permanecen en snapshots LXMI.

Backup nuevo e inmutable: `f4ea33c0c047d893319296420fd86784`, estrategia existente `descriptor_anchored_recursive_copy`. Manifest SHA-256 `182929afbefb7e08b226cb7e1651a44a66df95e55e12422c58752bf81ed40f80`. **VERIFIED**: 2445 entradas / 783 archivos / 285 directorios / 1377 symlinks, sin mismatches ni metadata unsupported. Source consistente durante captura; protected paths sin cambios observados. Restore en sandbox del mismo ID: **VERIFIED OFF-TARGET**, 2445 entradas / 783 hashes. Real-prefix restore **NOT TESTED**, executor no disponible. Backup histórico `f3551122468ed68b825a0182922f3be2` y manifest original permanecen intactos y verificados.

Source lógico 434363278 bytes; backup lógico igual, allocated 442916864 bytes. Espacio disponible antes/después de backup más prueba de restore: 28871073792 / 27955630080 bytes. Gate previo exige espacio para dos copias más reserva; no limpieza automática ni sustitución de la generación histórica.

## Validación nativa y resultado

**COMPROBADO — Tauri nativo:** Scan Steam; revisión semántica de 5 segundos; reconciliación; baseline actual; creación/verificación del backup; restore sandbox; reevaluación. UI muestra ambas generaciones, assessment actual y hashes/fingerprint redactados. AT-SPI limitado a la propia aplicación; no sustitución por navegador. Tauri/Vite propios cerrados al terminar. No consentimiento ni clicks de helper/executor/loader.

Una revisión expiró durante una espera de aprobación. El backend devolvió correctamente `steam_protected_semantic_review_required`; el TTL de 600 segundos no se amplió. Se renovó únicamente la revisión read-only, se capturó un baseline nuevo y se reevaluó el backup actual ya verificado, sin crear otra copia ni ejecutar helper. Resultado puntual final: **ELIGIBLE FOR REVIEW**. Esta evidencia no autoriza ejecución ni evita nuevas comprobaciones posteriores.

Se corrigió además el selector de journals: `recovery-source-drift-*` no debe interpretarse como `recovery-<backup_id>.json`. Test específico cubre convivencia de journals/reportes y elección de la generación actual.

Quality final: fmt/check/Clippy all-targets/all-features -D warnings PASS; cargo test workspace **353 PASS / 0 failed / 12 ignored**; frontend typecheck/lint/format:check/build PASS. Helper Windows sin modificaciones, reconstrucción ni ejecución. Tests normales sintéticos, sin prefix/Steam real.

**STOP:** se despejó el blocker de recuperación/estado protegido actual. Versión **0.8.7**, 0.8.8 sigue abierto. La siguiente entry exige capacidad de executor calificado en su sesión, preflight fresco y consentimiento independiente; no se realizan aquí. Sin 0.8.9/0.9, restauración real, Steam/game writes ni loader.

## Evidencia persistida

Solo JSON seleccionado/redactado, sin VDF raw, tickets ni account state:

| Artefacto | SHA-256 |
|---|---|
| [Revisión Steam](../06_Verificacion/evidencias/steam_protected_state_0_8_8_4.json) | `e6316bd8b36074b7059bd8cc9fdf2b68f7de7c6cdeb078c8c8e8166b08d30bee` |
| [Reconciliación actual](../06_Verificacion/evidencias/recovery_source_reconciled_0_8_8_4.json) | `36f034ffe3e29873a4df7f908bbf2f7660e6a0c27e4e324e04e17f6f142e4555` |
| [Baseline schema 2](../06_Verificacion/evidencias/prefix_baseline_current_0_8_8_4.json) | `afa1bc51daf4d767f31675f9d82d38de8808d2a1392e14d8ddd239d3c8703c05` |
| [Recovery actual](../06_Verificacion/evidencias/prefix_recovery_current_0_8_8_4.json) | `0edc24c94d0abe9f59281aa0e3810650744893b77cf6fbacbc4bd545feb9d317` |

[Recibo nativo](../06_Verificacion/evidencias/steam_protected_state_0_8_8_4_native.json): estado Reviewed, source acceptable, preflight Eligible, ambas generaciones visibles, source consistency y restore verified; helper/consent/Proton false.
