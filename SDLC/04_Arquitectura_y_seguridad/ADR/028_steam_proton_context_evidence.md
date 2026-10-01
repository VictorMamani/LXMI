# ADR-028 — Evidencia de contexto Steam / Proton

- **Estado:** Aceptada para LXMI 0.8.2
- **Fecha:** 2026-09-30
- **Ámbito:** discovery y readiness read-only de Zenless Zone Zero / Steam AppID `4162040`

## Contexto

LXMI detecta compatibility tools instaladas, manifest de juego, `compatdata/<AppID>` y `pfx`. Estos hechos no identifican por sí solos la herramienta que Steam usó para ejecutar el juego. La selección configurada tampoco demuestra el contexto efectivo de una ejecución concreta.

## Decisión

`lxmi-steam` inspecciona de forma read-only `config/config.vdf` desde el Steam root asociado con la biblioteca del manifest. Reutiliza Valve KeyValues con límite de tamaño y valida componentes de ruta. El mapping por AppID y la entrada global `0` se conservan en campos distintos. La herramienta configurada puede relacionarse con la lista de tools instaladas por su internal ID, pero no se copia a `GameRuntimePlan.selected` ni a `effective_runtime_selection`.

`compatdata`, `pfx`, runtime efectivo, prefix efectivo, contexto de proceso, relación con wineserver y requisitos same-prefix/same-wineserver son observaciones/capacidades separadas. En ausencia de observación de proceso, las relaciones efectivas permanecen `Unknown`. La readiness sigue `NeedsEvidence`, `LoaderStrategyDecision` sigue `NoDecision` y el gate no adquiere executor.

## Alternativas descartadas

- Elegir el único Proton instalado: la disponibilidad no demuestra selección.
- Elevar un mapping global `0` a selección de ZZZ: su alcance no equivale a una entrada por AppID y no se observó el proceso efectivo.
- Tratar la existencia de `compatdata/4162040/pfx` como prueba de runtime o prefix actualmente usado.
- Iniciar Steam/Proton/ZZZ o inspeccionar procesos para resolverlo: fuera del alcance read-only de 0.8.2.

## Consecuencias

La UI puede mostrar datos configurados, detected candidates y contexto desconocido de forma explícita. Cuando Steam no registra un override en el archivo inspeccionado, LXMI informa esa ausencia junto a la fuente consultada; no concluye cuál runtime se usará.

En el host consultado, el bloque `CompatToolMapping` de `config/config.vdf` estaba vacío; `compatdata/4162040` y `pfx` existían; Proton Experimental estaba instalado. El runtime efectivo, prefix efectivo, wineserver y requisitos del loader siguen sin determinar.

## Referencias

- [Evidencia Steam/Proton 0.8.2](../../01_Descubrimiento/evidencia_contexto_steam_proton_0_8_2.md)
- [Verificación LXMI 0.8.2](../../06_Verificacion/verificacion_0_8_2.md)
- [ADR-027 — Loader Strategy](027_xxmi_loader_strategy_for_lxmi.md)

## Extensión 0.8.3 — observación efímera de proceso

La incertidumbre de contexto que quedó tras 0.8.2 se aborda con una sesión pasiva iniciada mediante consentimiento explícito. El usuario inicia ZZZ manualmente desde Steam; un observer acotado a 90 segundos lee solo metadata de `/proc` de procesos cuyo UID coincide con el usuario actual. Solo para candidatos relevantes lee cmdline y claves de entorno allowlist. No lee memoria, no usa debug/ptrace, no envía señales y no controla procesos.

La evidencia se conserva solo en memoria, se redactan paths bajo el home al devolver el snapshot y se añade a readiness como `runtime_observation`. El summary identifica su estado de proceso y el timestamp. `GameRuntimePlan.selected`, mapping configurado por AppID/default global y candidates instalados permanecen campos independientes. Una observación acredita una sesión concreta, no una preferencia permanente de Steam.

El matching de `WINEPREFIX` en un proceso wineserver se informa únicamente como entorno observado coincidente. No demuestra que ZZZ se comunique con esa instancia, de modo que `wineserver_relationship` y requisitos same-wineserver siguen `Unknown`. Que Steam/proceso, Proton tool, compatdata o prefix se observen tampoco cambia Loader Strategy, compatibilidad del juego, ni el gate.

La UI no expone PID arbitrario ni selector de procesos. El backend parte del manifest Steam detectado para ZZZ, usa el ejecutable esperado del catálogo y rechaza evidencia de UID distinto. Si aparecen varios procesos posibles, devuelve `AmbiguousProcess` sin elegir uno. La sesión se detiene por usuario, por desaparición del proceso observado o al expirar su ventana; no se persiste environment, cmdline ni snapshot.

**Estado host 0.8.3 — LOCAL OBSERVED:** una sesión Tauri nativa observó `ZenlessZoneZero.exe` PID `691954`, AppID `4162040`, Proton Experimental build `experimental-11.0-20260924-x86_64`, compatdata coincidente, prefix observado y ascendencia relacionada con Steam. Un wineserver del UID actual expuso el mismo `WINEPREFIX`, pero no se concluye que el juego use esa instancia. La sesión tampoco selecciona permanentemente Proton ni cambia `GameRuntimePlan.selected`.

**Límite:** el snapshot compartido se capturó activo a los 67 segundos; no se registró su estado terminal. La UI reportó `PermissionDenied` para el entorno allowlisted del contexto del juego/ascendencia; el contador de candidatos directos fue cero porque no mide todos los procesos ascendentes. El runtime efectivo de esa sesión es conocido; la configuración persistente, el wineserver realmente usado por ZZZ y los requisitos del loader siguen separados/`Unknown`.

- [Diseño y límites de observación 0.8.3](../../01_Descubrimiento/observacion_runtime_zzz_steam_0_8_3.md)
- [Verificación 0.8.3](../../06_Verificacion/verificacion_0_8_3.md)
