# LXMI 0.8.8 — Harmless Real Prefix Entry (BLOCKED por UnexpectedMutation)

Última evidencia: 2026-10-02 UTC / 2026-10-01 America/La_Paz. Producto conserva **0.8.7**, checkpoint previo e84961b. **HOST TEST:** nuevo intento consentido en Tauri ejecutó el helper con handshake/nonce/hash/exit0. **BLOCKED / UnexpectedMutation:** cambios de contenido registry y metadata del prefix no aceptados por la policy. Protected paths sin diferencias dentro de cobertura. El intento anterior ProtectedPathChanged permanece como evidencia histórica. 0.8.9 no iniciado; 0.9 no autorizado.

## Arquitectura

Reutiliza helper Windows0.6/protocol1 y NativeProcessExecutor. El bridge general sigue rechazando cualquier modo no aislado. Solo una capability interna opaca validada por el camino de harmless entry permite `runinprefix` con compatdata4162040. No admite game executable/PID/DLL/path desde frontend. El helper permanece bajo helpers/; el nuevo marker JSON bajo tests/real-prefix-entry-v1/<nonce>/marker/, fuera de prefix y managed runtimes.

Tauri prepara revisión con preflight/recovery revalidado y baseline nuevo. Token de revisión aleatorio expira en600s; aceptación explícita por checkbox+botón del usuario. Backend rechaza consentimiento ausente/token ajeno/reutilizado; consume el intento antes de verificar/ejecutar, sin fallback o retry automático.

## Preconditions

Actividad same-UID0 y unresolved0; canonical UID/dev/inode corresponden a backup/revisión; versión exacta aprobada; source still matches verified recovery scope; backup y restore sandbox re-verificados; paths protegidos idénticos a revisión; baseline completo; >=256MiB disponibles. No runtime sustituto. Preflight debe ser EligibleForReview.

Environment construido desde cero: PATH/LANG del host; HOME y XDG config/cache/data privados de sesión; STEAM_COMPAT_CLIENT_INSTALL_PATH descubierto y STEAM_COMPAT_DATA_PATH real; marker no secreto y WINEDEBUG. No copia de ZZZ env, SteamAppId/SteamGameId, DLL overrides, Wine server/debug commands o launch options. Proton deriva su WINEPREFIX del compatdata. Exact command: programa Proton descubierto, argumentos `runinprefix` y helper LXMI fijo. No shell.

## Fuente local Proton — READ-ONLY HOST

Instalado `1790244902 experimental-11.0-20260924-x86_64`. Script SHA-256 `077db31dda75c570c1a91014f9a814b785307bc91e5dfb1480b984dc34166a54`. Archivo local `Proton - Experimental/proton`, líneas2180–2220: runinprefix llama init_session(False), omite setup_prefix en el condicional2073–2074, pero ejecuta init_wine, fixups/default-prefix checks y sync_appcache. Esta última retorna sin SteamGameId (1275–1278). No equivale a operación read-only de Wine/Proton ni se extrapola a otra build.

Referencia primaria consultada [Valve Proton script experimental_11.0](https://github.com/ValveSoftware/Proton/blob/experimental_11.0/proton). Branch no fijado como commit en esta revisión: API commit no disponible mediante browser; conclusión ejecutable respaldada por bytes/hash del script local aprobado. No se ejecutó Proton para investigarlo.

## Response y límites

Handshake exige protocolo/version/helper hash/nonce/environment marker/Windows path readable/hash marker/exit0, reutilizando comprobaciones0.6. Timeout20s, stdout/stderr acotados1MiB. La ruta real no hace group kill: nunca global wineserver. Ante timeout se termina únicamente launcher child propio; si un helper/servidor continúa, actividad unresolved/activa mantiene gate cerrado. No se garantiza cleanup de Windows descendants que Wine reparentó. No se llama wineserver cleanup ni rollback real.

## Snapshots / mutations

Antes: baseline selectivo0.8.7 con hashes y metadata, incluido registry/dosdevices/protected paths. Después: misma policy, incluso si alguna fuente no está disponible; resultado y detalle se preservan en journal privado. Espera natural acotada10s; inactivity checks puntuales, no freeze/lock ni prueba continua de ausencia de una carrera externa.

Registry metadata distinta con idéntico content SHA se clasifica ExpectedWineMutation (compatible con flush, causa no demostrada). Cambios de contenido registry son PossibleWineMutation y **bloquean**; mapping/estructura/otros cambios son Unknown y bloquean. Cualquier protected-path mismatch bloquea sin atribuir automáticamente su causa a LXMI. No se afirma whole-disk/whole-game coverage. No rollback automático.

## Gate secuencial

Solo Verified o VerifiedWithExpectedMutations más handshake/backup/identidad/runtime/inactividad/protected intactos permiten revisar cierre0.8.8. Ningún resultado actual resuelve loader/game compatibility; strategy NoDecision y game gate cerrado. No comenzar0.8.9 antes de host/native PASS y quality/version/checkpoint0.8.8.

## Conclusión host — LOCAL OBSERVED / BLOCKED

El usuario aceptó consentimiento en la ventana nativa y pulsó Run Harmless Prefix Entry Test. La respuesta fue `protected_path_changed`. El backend revisa este campo antes de construir la capability/marker y antes de execute_harmless_prefix_entry; **Proton/helper no ejecutados**. No hay receipts real-prefix-entry-* ni marker de esta operación. No se reintenta ni se restaura nada. Tauri/Vite cerrados tras recoger evidencia; ningún proceso Steam/Wine/juego controlado.

Baselines privados existentes comparados read-only:

| Evidencia | Review | Preflight inmediato |
|---|---|---|
| Manifest | zzz-prefix-70a8779973fde696036eabd53ce9067a.json | zzz-prefix-3207b5b9493c2beaf767c6b02dd98c88.json |
| UTC | 2026-10-01 07:11:35 | 2026-10-01 07:16:50 |
| SHA-256 manifest | 1fee28ab92fc2bd41093924117952549db760d2ebf08ab9bc0540eaa8a54159a | cadb530f7c164c9173e9bfb8de2d5038c815f94552cd61d85807a86465d1529c |

Un único protected path distinto: `~/.local/share/Steam/userdata/1416423935/config/localconfig.vdf`.

| Campo | Antes | Después |
|---|---|---|
| File SHA-256 | f3fcbabfb6588f6fcfa04c802da7fcc371ed7a9c136a5c1db3670b776d009ef3 | fd0cc730aadc2d0175083ef97229db2ca729e7483ba21de6b96d3089691190b2 |
| Size | 59838 | 59838 |
| Inode | 10879547 | 10879325 |
| Mtime seconds/ns | 1790837972 / 672723731 | 1790838888 / 729909082 |

Los otros protected paths seleccionados son idénticos, igual que todo selected_prefix_paths y la identidad UID1000/dev66306/inode11538172. Esto prueba una diferencia anterior a ejecución, **no** qué clave Steam cambió o quién escribió. No se inspecciona/persiste contenido config, no se afirma launch options intactas del archivo cambiado. Escritura de background Steam es una posible explicación, no hecho atribuido.

Evidencia de diagnóstico temporal `/tmp/lxmi-088-blocked-diagnosis.json`; los manifests fuente permanecen bajo snapshots LXMI. El consentimiento fue recibido, pero no autoriza sustituir/ignorar evidencia que cambió. No relajar la protección ni renovar baseline automáticamente para forzar PASS.

**Resultado final del intento: ProtectedPathChanged / BLOCKED BEFORE EXECUTION.** Version0.8.7; 0.8.8 no cerrado como successful prefix entry; 0.8.9 NO INICIADO; 0.9 no autorizado. La próxima decisión requiere investigar la estabilidad/política de los archivos Steam protegidos mediante trabajo read-only y una instrucción nueva antes de cualquier intento real. Helper protocol/hash/nonce in real prefix: NO COMPROBADO.

## Investigación posterior autorizada, sin reintento

[Estabilidad localconfig](estabilidad_localconfig_preflight_0_8_8.md):90s read-only,1 cambio semántico fuera de rama ZZZ; rama ZZZ estable y LaunchOptions actual no presente. Writer/causa históricos UNKNOWN. Guard full-file intacto; diagnóstico mejorado para mostrar path/campos sin valores. No helper ni0.8.9 ejecutados.

## Nuevo intento nativo consentido — HOST TEST / STOP

El usuario volvió a revisar y consentir la prueba desde Tauri. Sesión `699f4768f21d30842653eb404baeb237`. No se aceptó consentimiento mediante herramientas del agente. El preflight inmediato de esta sesión pasó; no se relajó el guard de localconfig ni se reutilizó el consentimiento del intento anterior.

| Evidencia privada LXMI | SHA-256 |
|---|---|
| snapshots/real-prefix-entry-699f4768f21d30842653eb404baeb237-before.json | 6c44ae8fd97b8ef55348a0b789b48fb1138a8d7b279fe6d6be1fa66d497b3926 |
| snapshots/real-prefix-entry-699f4768f21d30842653eb404baeb237-after.json | 1f2b3d6e7bdaecdc15f14a40a1ebfd3ddae62a387e493f6cfa1ce802fc9da246 |

Baseline antes:2026-10-02T02:58:30Z. Después:02:58:32Z. Recibo final escrito02:58:39Z. Las horas corresponden a 2026-10-01 por la noche en America/La_Paz.

**COMPROBADO / HOST TEST:** helper fijo0.6.0, SHA esperado/real `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2`; protocolo1, handshake succeeded, nonce del stdout coincide con sesión y marker/hash verificados. Marker externo leído desde Z:, digest `a6190d02e7949d00cd5e7aaf667e0ca2f6bdc28c8d7e599eef2ae27e34dbbbd2`. Proton `runinprefix` + helper, argv directo; exit0/1426ms, output no truncado, stderr únicamente diagnóstico fsync. No game/loader command.

Identidad prefix antes/después UID1000/device66306/inode11538172; runtime SameBuild exacto aprobado. Actividad0/unresolved0 en ambos snapshots puntuales; esto no es prueba continua contra carreras. Backup original continúa Verified, no sobrescrito. **READ-ONLY evidence verification:** manifiestos existentes releídos, nonce/exit/protected equality comprobados; no nueva ejecución.

### Mutaciones observadas dentro de cobertura

| Path relativo a pfx | Diferencia | Clasificación aplicada |
|---|---|---|
| raíz pfx/ | mtime y fingerprint de entradas inmediatas; root inode estable | Unknown |
| dosdevices/ | mtime y recreación metadata/inodes de com1..com32; mismos destinos observados | Unknown |
| system.reg | 3996760 → 4027760 bytes, hash/contenido/inode/mtime distintos | PossibleWineMutation |
| user.reg | 501382 → 516931 bytes, hash/contenido/inode/mtime distintos | PossibleWineMutation |
| userdef.reg | 4190 bytes y contenido hash igual; inode/mtime distintos | ExpectedWineMutation |

Mappings c:, s:, z: y targets COM observados son iguales. Igual destino no convierte recreación de symlinks ni diferencias registry en cambios aceptados. No se dereferenciaron devices ni se abrió contenido de registros para investigar valores privados. Fingerprint de raíz incluye metadata de registros/directorio: las filas están relacionadas y no se cuentan como mutaciones independientes de contenido.

| Registro | SHA antes | SHA después |
|---|---|---|
| system.reg | d3d6ea93c4a5bc9159fd8dbfa9e234ace5d9527535326945e2e32f8aedb32fbc | c39a454cf90dcc033af5cc68a6b7d8b5defef22d3c4d48a12466127b97de08a5 |
| user.reg | 2ba0a3b31e0050a1614eb61dcef49e240eea9bcb83c5394fe9e9adede04c08c2 | 3b49f379e52e1d4c751e260ef5252647568f10a582f10281764567ab742134ea |

**COMPROBADO:**9 registros de paths protegidos idénticos antes/después: Steam config, appmanifest, executable game, metadatos game root/userdata/packages/runtimes, y2 localconfig. No equivale a hash completo de instalación, managed packages o launch options efectivas fuera de fuentes monitorizadas. No se afirma whole-disk coverage.

**UNKNOWN:** cambios semánticos concretos dentro de system.reg/user.reg y explicación operativa completa. El nombre PossibleWineMutation no constituye aprobación. El snapshot before conserva hash/metadata, no copia del contenido de cada registro de esa hora; backup anterior es recuperación, no se declara diff semántico exacto de esta ejecución por su mera existencia.

### Conclusión y UI

**Resultado global: UnexpectedMutation / BLOCKED.** Helper entry verificada técnicamente, pero gate0.8.8 **NO PASS**. Se aplica STOP del master; no0.8.9/0.9, no version bump/checkpoint de incremento completado, no segundo intento ni restore automático. Backup se conserva inmutable, real restore NotTested, loader strategy NoDecision y compatibilidad ZZMI/ZZZ UNKNOWN.

Tauri nativo inspeccionado read-only por AT-SPI: muestra `Result: unexpected_mutation`, panel `Before / after mutations`, `Verified unchanged`, `Unchanged within monitored coverage` y `Verified; original backup preserved`. El texto compartido por el usuario corresponde al estado previo a la ejecución, no al recibo final. No hay evidencia de que el resultado se haya perdido en el wiring; no se cambió el diseño ni se añadió un botón nuevo.

Antes de considerar un siguiente intento hace falta una investigación separada de las mutaciones observadas y del alcance de recuperación. No volver a capturar baseline/backup para hacer desaparecer el fallo ni reclasificar diferencias a Expected sin evidencia.
