# ADR-027: estrategia de loader XXMI para LXMI

- **Estado:** `NoDecision` para ZZMI/ZZZ/Linux; Direct Inject solo está verificado en el laboratorio LXMI.
- **Fecha:** 2026-09-28; reevaluada 2026-09-29 en LXMI 0.8.1.
- **Decisión:** No promover automáticamente Direct Inject porque el fixture pasó. Upstream ofrece Hook como default de ZZMI y Direct Inject como modo de Custom Launch. No existe evidencia de un requisito técnico Hook-only de ZZMI, pero su ausencia no demuestra compatibilidad de Inject. El Hook Lab sigue fail-closed y no ejecuta Hook mientras el scope del `3dmloader.dll` no pueda restringirse a un solo proceso. No se selecciona una estrategia de producción para ZZMI/ZZZ.

## Contexto y evidencia

XXMI Launcher v2.2.1 está fijado a commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`. En su integración ZZMI, `use_hook` hereda el valor predeterminado `true`; la ruta observada configura `DllInjector` con `3dmloader.dll` y usa su API de Hook. La rama alternativa `use_hook = false` llama el export `Inject` para el proceso objetivo. Las notas de v2.1.5 registran que se sustituyó `pyinjector` por un injector propio en `3dmloader.exe` para el modo Direct Inject basado en `WriteProcessMemory`; no se extrapola ese cambio a la ruta default ZZMI.

La traza 0.8.1 encontró en `model_importer.py` que Custom Launch asigna `use_hook = (custom_launch_inject_mode == 'Hook')`. En consecuencia, Custom Launch + `Inject` selecciona el Direct Inject genérico del mismo `MigotoInjector`; en cambio, el start estándar propaga el `true` heredado por ZZMI. `Bypass` también usa la rama `use_hook=false`, pero `is_xxmi_dll_used()` deja fuera `d3d11.dll` a menos que se añada explícitamente entre Extra Libraries. El Direct Inject genérico inicia el comando configurado y después busca el target por nombre/PID hasta timeout antes de llamar al export `Inject` de `3dmloader.dll`.

El alcance de Hook se reexpresa de forma más precisa: el código Wine consultado almacena hooks globales en la tabla del desktop actual, administrada por el wineserver; no prueba el comportamiento de Proton ni del hook del `3dmloader.dll` concreto. `WINEPREFIX` distinto permite sesiones Wine independientes según la documentación general de Wine. El alcance exacto en Proton y los requisitos de same-prefix/wineserver para ZZZ permanecen `Unknown`.

LXMI probó únicamente el export `Inject` del `3dmloader.dll` oficial XXMI Libraries v1.1.7 (`6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9`). Release y componente se verifican en el store; licencia revisada para uso local de prueba, no redistribución. El host test se limitó al hijo creado por `CreateProcessW` en el mismo runner de LXMI. El target, test DLL, runner, nonce, runtime, marker y timeout son fijos y validados en backend.

## Opciones

| Estrategia | Evidencia | Decisión |
|---|---|---|
| Default ZZMI Hook mediante `3dmloader.dll` | `use_hook=true` heredado; Hook se prepara antes del target. Código Wine describe hooks globales por desktop, sin host test Proton. | **BLOCKED** por la política del laboratorio target-only; no prueba que ZZMI requiera Hook. |
| Direct Inject mediante export `Inject` de `3dmloader.dll` | Host test positivo/negativo contra un proceso de prueba propio. XXMI lo ofrece desde Custom Launch + `Inject`; la rama estándar ZZMI sigue Hook. | Candidate para estudiar; **NoDecision** para ZZMI/ZZZ/Linux/Proton. |
| `3dmloader.exe` | Release v2.1.5 documenta un injector propio para una modalidad Direct Inject; LXMI no lo ejecutó. | No seleccionado. |
| XXMI Launcher portable bajo Wine | Upstream ofrece edición portable; mantiene un launcher/UI Windows que LXMI busca sustituir. No valida el contexto ZZZ Steam/Proton. | No seleccionado. |
| Injector/helper propio | Añadiría superficie de proceso e implementación de técnicas de carga. | No implementado. |

## Restricciones de seguridad

- El único target admitido es el PE de test de LXMI bajo `tests/loader-v1/target/`; el backend vuelve a canonicalizarlo y rechaza rutas de Steam/juego y nombres ajenos.
- El runner crea el target y pasa al export solo el PID devuelto por esa creación. No acepta PID externo, process browser, nombre o DLL arbitrarios; no usa búsqueda global ni fallback.
- El test DLL verifica el basename del proceso anfitrión y el nonce generado por LXMI antes de producir marker. El marker se vuelve a validar contra proceso, nonce, rutas Windows observadas y respuesta del runner.
- Timeout, salida acotada, process group del hijo y prefix de Proton están limitados a LXMI. No hay anti-cheat, ZZZ, Steam o HoYoPlay en el experimento.
- El backend rechaza `LoaderExperimentMode::Hook` antes de consultar rutas o ejecutar procesos (`HookScopeNotIsolated`). Los tests verifican ese fail-closed, no una ejecución de Hook.
- `3dmloader.exe`, ZZZ, cambios de launch options, Steam, prefix de juego y toda integración real quedan fuera de esta ADR.

## Consecuencias

El resultado 0.7 solo demuestra que Direct Inject carga el fixture bajo la build Proton probada. La revisión 0.8.1 fija que upstream también ofrece la ruta Inject desde Custom Launch y que ZZMI no declara Hook como requisito exclusivo; no demuestra que la combinación ZZMI/ZZZ funcione con Inject bajo Proton. La evidencia Wine acota un Hook global al desktop de su sesión/server, pero no demuestra el comportamiento de Proton/3dmloader ni un aislamiento target-only.

LXMI 0.8.1 añade `LoaderStrategyDecision`, evidencia de aislamiento, criterios separados para candidato Direct Inject y componentes de gate granulares. La selección Proton del juego permanece `Unknown`; la decisión queda `NoDecision` y readiness `NeedsEvidence`, no `ReadyForControlledExperiment`. LXMI 0.9 no puede comenzar. El siguiente trabajo propuesto es LXMI 0.8.2, para recoger evidencia faltante de proceso/runtime Steam-Proton sin lanzar ZZZ ni ejecutar Hook; el Hook Lab no pierde su guard.

### Seguimiento 0.8.2

La inspección read-only encontró vacío el `CompatToolMapping` local para AppID `4162040`; las tools instaladas no determinan qué runtime usaría Steam. `compatdata/4162040/pfx` se conserva como candidato filesystem, no prefix efectivo. `LoaderStrategyDecision` sigue `NoDecision`, readiness sigue `NeedsEvidence` y las relaciones loader/prefix/wineserver siguen `Unknown`. No se ejecutó Hook, Direct Inject, Steam, Proton, Wine o ZZZ. Ver [evidencia de contexto 0.8.2](../../01_Descubrimiento/evidencia_contexto_steam_proton_0_8_2.md) y [verificación](../../06_Verificacion/verificacion_0_8_2.md).

## Referencias

- [XXMI Launcher v2.2.1 fijado](https://github.com/SpectrumQT/XXMI-Launcher/tree/d56786b8dacb00c35204bff45ff5b8b83bd8962a)
- [Notas XXMI Launcher v2.1.5](https://github.com/SpectrumQT/XXMI-Launcher/releases/tag/v2.1.5)
- [XXMI Libraries v1.1.7 fijado](https://github.com/SpectrumQT/XXMI-Libs-Package/tree/6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9)
- [Release XXMI Launcher v2.2.0: WWMI cambia a Direct Inject](https://github.com/SpectrumQT/XXMI-Launcher/releases/tag/v2.2.0)
- [Implementación Wine de hooks por desktop](https://github.com/wine-mirror/wine/blob/master/server/hook.c)
- [Documentación Wine de `WINEPREFIX` y sesiones](https://github.com/wine-mirror/wine/blob/master/server/wineserver.man.in)
- [Investigación del loader](../../01_Descubrimiento/loader_xxmi.md)
- [Estrategia ZZMI Linux 0.8.1](../../01_Descubrimiento/estrategia_loader_zzmi_linux.md)
- [Readiness ZZMI / Steam / Linux](../../01_Descubrimiento/launch_zzmi_steam_linux.md)
- [Verificación de LXMI 0.7](../../06_Verificacion/verificacion_0_7.md)

### Seguimiento 0.8.5 — límite de identidad del fixture

Same-prefix: Direct Inject del componente oficial vuelve a cargar la DLL propia con marker/nonce. Cross-prefix: ambos ven filesystem pero el pipe de identidad no cruza; se observaron IDs Windows iguales en target y runner de contexts distintos. El guard impide entregar a upstream un PID no correlacionado. `TargetIdentityUnavailable` no acredita un error de `Inject`, ni same-prefix como requisito universal. **NoDecision** permanece; Hook sigue bloqueado; 0.9 no está autorizado. Ver [investigación 0.8.5](../../01_Descubrimiento/direct_inject_context_boundary_0_8_5.md).

### Seguimiento 2026-10-15 — evidencia de ZZZ e isolation Hook en fixture

Direct Inject contra ZZZ real cargó el módulo, pero con el juego ya iniciado
no hubo efecto visual y a 1.6–2.8 s del arranque el proceso terminó con
`0xc0000409` (Libraries 1.1.7 y 1.2.2). El control sin inyección llegó a
«Pulsa para entrar». Upstream ofrece Hook como default de ZZMI precisamente
para cargar antes de crear el dispositivo.

El fixture `hook_isolation.py` ejecutó el `HookLibrary` oficial en prefixes
propiedad de LXMI. El hook global quedó dentro de su wineserver, los procesos
ajenos rechazaron ZZMI en `DllMain`, el target lo recibió antes del dispositivo
(dispositivo envuelto, Present correcto) y `UnhookLibrary` detuvo nuevas cargas.
Si el target resuelve `d3d11` antes de su primera ventana, ZZMI sale
deliberadamente con `0xc0000135`.

**Hook pasa de BLOCKED a candidato con evidencia de aislamiento en fixture.**
El guard de producción del Loader Lab no cambia. `LoaderStrategyDecision`
sigue `NoDecision` hasta una sesión ZZZ consentida en modo Hook
(`experiment.py run --arm-before-game --hook`). Ver la
[verificación](../../06_Verificacion/verificacion_primera_carga_zzz.md) y el
[resumen](../../06_Verificacion/evidencias/aislamiento_hook_fixture_20261015.json).

### Resolución 2026-10-15

La sesión ZZZ en modo Hook cargó ZZMI con el juego estable y mods visibles en
partida. Para ZZMI/ZZZ en Steam + Proton la estrategia queda decidida en
[ADR-032](032_jugar_con_mods_hook_oficial.md). Esta ADR conserva el historial.
