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

## Extensión 0.8.4 — fixture de contexto, sin join del juego

Dos peers propios compartieron un evento con mismo prefix y no lo compartieron con prefixes diferentes usando Proton constante. Ambos pudieron leer el mismo marker mediante Z. Mappings completos y mapping de ese marker se comparan aparte. Esta evidencia se añade al laboratorio, no a una configuración persistente de Steam ni a una selección del planner.

El wineserver se mantiene Unknown como identidad/endpoint: UID y WINEPREFIX coincidentes solo producen candidatos efímeros. La observación de ZZZ 0.8.3 se compara documentalmente; no se vuelve a lanzar el juego ni se ejecuta helper en 4162040. No se establece aún un requisito de contexto para un loader ni se cambia ADR-027/gate. [Investigación 0.8.4](../../01_Descubrimiento/contexto_wine_proton_0_8_4.md), [ADR-029](029_windows_runtime_context_probe.md).

### Seguimiento 0.8.5 — PID y contexto no se colapsan

La identidad de tool, compatdata, prefix, mapping, wineserver y PID sigue independiente. Un PID Windows observado en un prefix no es una identidad global reutilizable en otro: el control tuvo PID 220 para target y runner simultáneamente en contexts distintos. El handshake de pipe precede cualquier acceso query-only al target y la llamada upstream. La DLL se verificó únicamente en el fixture same-prefix; no se usó el prefix del juego. GameContextJoinPlan incorpora evidencia del fixture como dato declarativo, con guard y blockers intactos. Cross-prefix Inject permanece **NO COMPROBADO**.

### Extensión 0.8.5.1 — binding explícito, sin join real

El pipe dejó de ser requisito de identidad: self report de fixture propio con nonce/ruta por sesión/Windows PID/FILETIME se correlaciona con el child Linux del launcher y se consulta ese único PID desde el runner con query limitada. Same-prefix verifica imagen y creación y permite Inject; en cross-prefix OpenProcess falla Win32 87 sin handle y se bloquea antes de cargar upstream. Esto reduce incertidumbre de la topología fixture; no prueba imposibilidad universal ni endpoint wineserver.

Se preserva PID 220 simultáneo de contexts separados como evidencia histórica y regresión de colisión. Windows PID y Linux launcher PID siguen distintos. La consulta no enumera ni descubre procesos, y no sustituye observación de Steam por selección del planner. Requisitos generales quedan NotEstablished, GameContextJoinPlan declarativo, ADR-027 NoDecision y gate del juego cerrado. [Cierre verificado](../../06_Verificacion/verificacion_0_8_5.md).

## Evidencia 0.8.6 — reentrada fixture independiente

Same-prefix: A publica ready, helper B se inicia mediante nueva invocación Proton runinprefix y verifica evento más PID/image/creation/nonce. Control different-prefix no verifica evento ni identidad; el PID homónimo consultado corresponde al propio helper. Filesystem compartido permanece señal separada.

Se añade PrefixFileIdentity (canonical/dev/inode/UID) y WineServerEndpointEvidence: ruta DERIVED según Wine fijado, socket metadata LOCAL OBSERVED. No se eleva socket a PID de server ni conexión exacta del peer. Alias a mismo inode tienen interpretación derivada, no host test.

GameContextJoinPlan incorpora VerifiedForFixture únicamente; game prefix entry NotTested, riesgo/reversibilidad UNKNOWN, backup/consentimiento requerido. No ejecutor de join, selección de Proton del juego intacta. Se mantiene ADR-027 NoDecision y Hook/gate cerrados. No se crea otro ADR-029: el existente ya documenta el probe, aquí se extiende evidencia. [Verificación](../../06_Verificacion/verificacion_0_8_6.md).
