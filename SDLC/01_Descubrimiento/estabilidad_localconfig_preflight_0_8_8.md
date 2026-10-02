# LXMI0.8.8 — investigación read-only de estabilidad localconfig

Fecha2026-10-01. Continúa el diagnóstico del intento ProtectedPathChanged; **no reintento**, helper/Proton/Wine/Steam no ejecutados ni controlados. Producto0.8.7, 0.8.9/0.9 no iniciados. Se conservan los guards y la cobertura de archivos protegidos.

## Evidencia histórica

Los manifiestos de review y preflight inmediato conservan hash y metadata, no contenido del VDF. Prueban que `userdata/1416423935/config/localconfig.vdf` cambió hash/mtime/inode, pero no qué clave cambió ni quién escribió. No existe archivo localconfig anterior adyacente observado que permita reconstruir ese diff. Véase [diagnóstico original](harmless_real_prefix_entry_0_8_8.md). **UNKNOWN:** campo/autor/causa históricos.

## READ-ONLY HOST — observación acotada

Primeros3 samples durante10s: mismo hash `fd0cc730aadc2d0175083ef97229db2ca729e7483ba21de6b96d3089691190b2`, inode10879325, size59838. Esto solo establece estabilidad durante esas muestras, no estabilidad permanente.

Después, probe90s, cada2s, usando parser KeyValues existente lxmi-steam. Lectura regular same-UID bounded2MiB, O_NOFOLLOW/O_NOATIME, verificación metadata antes/después; valores únicamente en memoria. No dump/persist de configuración. Digests comparan estructura case-normalized; duplicados scalar se rechazan. El probe es diagnóstico, no interpretación contractual de Steam ni gate de ejecución.

**LOCAL OBSERVED:** a elapsed28s cambió el archivo;1 evento en90s. Hubo1 leaf semántico distinto, fuera de la rama `UserLocalConfigStore/Software/Valve/Steam/apps/4162040`. Hash de esa rama permaneció igual; entry LaunchOptions no presente en la rama actual antes/después. No se imprimieron valores ni la clave privada concreta de otras ramas, solo categoría `Other user configuration branch`.

| Evidencia | Inicial | Final inspeccionado |
|---|---|---|
| Hash archivo | fd0cc730aadc2d0175083ef97229db2ca729e7483ba21de6b96d3089691190b2 | c4a0fe1629bd377310c912738387b04f18d23ced0a30229fa9b6ca547faf3dae |
| Inode |10879325|10879922|
| Mtime seconds/ns |1790838888 /729909082|1790839706 /33408008|
| Size |59838|59838|

Digest semántico inicial documento `f8180c745f07ba33df23bc07c1bb367fd2e3c4dff0444c0d8fc96e49457995c0`. Digest rama ZZZ estable `680cadb0b67897b4feda8f7e32209c81f79adbd40ade3f6b433f2b41ca95a9d2`. Valores concretos no guardados. Ausencia LaunchOptions en esta rama actual no demuestra ausencia histórica ni descarta otra fuente/setting Steam efectiva.

## Process evidence y causalidad

Checks puntuales read-only current UID: comm steam0; segundo check independiente de comm, exe basename steam/steamwebhelper0. No process list/general browser ni environment dump. No se persisten PIDs. Esto significa **no detectados en esos instantes**, no prueba que Steam no existiera en todo el intervalo o que otro proceso no pudiera escribir. El cambio no fue causado por el probe: solo usa lectura (O_NOATIME en el probe semántico), ningún write syscall sobre VDF. No se usó strace/ptrace/inotify de terceros ni se inspeccionó memoria para atribuir writer.

**UNKNOWN:** autor y semántica concreta de la clave externa; archivo es contenido mutable, no un blob inmutable garantizado. No se atribuye automáticamente a heartbeat/flush Steam. La consulta primaria Steamworks no proporcionó contrato del schema localconfig ni lista de campos seguros volátiles. [Steamworks documentación](https://partner.steamgames.com/doc/sdk/uploading). No se usan foros/implementaciones terceras como prueba de semántica oficial.

## Efecto en el preflight

La política actual protege el **archivo completo**, no solo la rama del juego. Por tanto una variación externa a4162040 puede bloquearla aunque esa rama permanezca igual. El resultado es coherente con esa política; no es un fallo del hash ni permiso para ignorar la variación. El cambio externo también puede ser relevante a settings globales desconocidos.

**Decisión de esta investigación:** mantener full-file guard intacto. No se elimina localconfig de ProtectedGamePaths, no se admite cualquier cambio externo aZZZ, no se refresca baseline automáticamente para conseguir PASS y no se reutiliza consentimiento consumido. Se mejoró únicamente diagnóstico de error: path y campos que cambiaron, sin valores de config/hash privados. Test sintético cubre metadata/hash/path-set y no disclosure.

Para una revisión posterior: capturar diffs semánticos en memoria al iniciar revisión, manteniendo bloqueo por schema/duplicados/cambios desconocidos; identificar y justificar una allowlist de metadata no operativa antes de proponer una policy distinta. Alternativamente, el usuario podría mantener Steam cerrado y esperar estabilidad antes de una revisión nueva; **no se ha demostrado que eso baste**, pues los checks puntuales anteriores no detectaron Steam y aun así se observó cambio. Ningún consejo autoriza ahora nueva ejecución.

## Reproducibilidad / límites

[Probe fuente](../05_Construccion/aplicacion/tools/observe-localconfig-readonly.rs), herramienta host manual independiente, no importada por Tauri ni cargo test. Target local fijo de esta investigación, no acepta frontend PID/path. Reutiliza parser y SHA de dependencies ya compiladas; rustc manual con --extern lxmi_steam/sha2/libc y -L dependency=target/debug/deps. Se ejecutó como bin Linux read-only bajo /tmp, no helper Windows. La primera ejecución duró90s; el probe actual tiene un límite de240s. No se instala dependency/otro runtime. La ventana bounded no prueba frecuencia ni periodicidad de escrituras.

0.8.8 sigue **BLOCKED / helper NO COMPROBADO**. Cambios sin commit de fase cerrada. El master se detiene antes de0.8.9. La autoría histórica no puede recuperarse de hashes; requeriría una fuente de evidencia adicional que no exceda la frontera read-only.

## Seguimiento ampliado — misma frontera read-only

Dos ventanas de240s solapadas detectaron el mismo evento: elapsed172s en la primera y34s en la segunda. No son dos escrituras independientes. Cada ventana terminó con1 evento. **LOCAL OBSERVED:**1 leaf cambió bajo `UserLocalConfigStore/<redacted-key>`; fuera de `Software` y de la rama ZZZ. Esta rama siguió con digest `680cadb0b67897b4feda8f7e32209c81f79adbd40ade3f6b433f2b41ca95a9d2`; no cambió la presencia de LaunchOptions en ella.

Estado posterior muestreado: inode10879912, mtime1790840537/95884380, hash archivo `a5c0851cd7c93a23a0addef886500f6e17c7e3bd0feb65006874f7f9cc4bfab6`, digest semántico `8b7e05a59ac6a9f3ee4eefa10c384243eaee82590d1649c3b7564cf6b4078503`. Se inspeccionaron nombres de campos de schema de primer nivel sin imprimir valores. No se identificó inequívocamente cuál cambió en las ventanas ya concluidas: la allowlist de diagnóstico lo redactó. No se conservó el contenido anterior. No se clasifica como timestamp inocuo ni como cambio de launch options.

**UNKNOWN:** nombre exacto del campo cambiado, significado operativo, escritor y vínculo causal con el intento histórico. La evidencia permite localizar el bloqueo al archivo completo protegido; no permite autorizar una excepción. Una futura modificación de política necesitaría justificar explícitamente qué campos se pueden tolerar y rechazar los restantes. Esta investigación no modifica esa política.

Última ventana independiente240s:0 cambios detectados. La ampliación de nombres permitidos de schema no produjo una nueva diferencia que permita identificar la clave. El probe terminó normalmente; no queda scanner persistente. **LOCAL OBSERVED:** estabilidad limitada a esa ventana; no garantía de estabilidad futura.
