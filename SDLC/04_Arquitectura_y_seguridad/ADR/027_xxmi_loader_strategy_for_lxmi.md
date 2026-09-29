# ADR-027: estrategia de loader XXMI para LXMI

- **Estado:** aceptada para Direct Inject en el laboratorio; decisión de producción `NoDecision`.
- **Fecha:** 2026-09-28; actualización LXMI 0.8.
- **Decisión:** Direct Inject permanece permitido únicamente para el test target fijo de LXMI. El Hook default de ZZMI queda bloqueado en el Loader Lab mientras no se pruebe que su alcance puede limitarse con seguridad al proceso de test. No se selecciona una estrategia de producción para ZZMI o ZZZ.

## Contexto y evidencia

XXMI Launcher v2.2.1 está fijado a commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`. En su integración ZZMI, `use_hook` hereda el valor predeterminado `true`; la ruta observada configura `DllInjector` con `3dmloader.dll` y usa su API de Hook. La rama alternativa `use_hook = false` llama el export `Inject` para el proceso objetivo. Las notas de v2.1.5 registran que se sustituyó `pyinjector` por un injector propio en `3dmloader.exe` para el modo Direct Inject basado en `WriteProcessMemory`; no se extrapola ese cambio a la ruta default ZZMI.

LXMI probó únicamente el export `Inject` del `3dmloader.dll` oficial XXMI Libraries v1.1.7 (`6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9`). Release y componente se verifican en el store; licencia revisada para uso local de prueba, no redistribución. El host test se limitó al hijo creado por `CreateProcessW` en el mismo runner de LXMI. El target, test DLL, runner, nonce, runtime, marker y timeout son fijos y validados en backend.

## Opciones

| Estrategia | Evidencia | Decisión |
|---|---|---|
| Default ZZMI Hook mediante `3dmloader.dll` | Upstream lo prepara antes de iniciar el target. `HookLibrary` no recibe nombre del target; `WaitForInjection` sí lo recibe para observar el resultado. El comentario upstream lo identifica como hook global. La ruta no fue ejecutada en LXMI 0.8 por el límite single-target. | **BLOCKED** por la política del laboratorio; comportamiento bajo Proton **NO COMPROBADO**. |
| Direct Inject mediante export `Inject` de `3dmloader.dll` | Host test positivo/negativo contra un proceso de prueba propio; el componente viene de Libraries v1.1.7 autenticado. | Usada solo en Loader Lab, nunca contra juego. |
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

El resultado 0.7 solo demuestra que Direct Inject carga el fixture bajo la build Proton probada. La revisión 0.8 describe el orden del Hook upstream, pero no demuestra su compatibilidad con Proton ni que el hook afecte exclusivamente al target. Tampoco determina si ZZZ Steam es compatible o si loader y juego deben compartir prefix.

LXMI 0.8 añade un `ZzmiLaunchReadiness` declarativo y conserva la selección Proton del juego como desconocida. Dado que Hook está bloqueado/no probado y siguen pendientes el contexto Steam/Proton, same-prefix y compatibilidad específica, el gate no autoriza un experimento ZZZ. LXMI 0.9 no puede comenzar; primero se requiere una vía de investigación/aislamiento aprobada que no permita carga global en procesos ajenos.

## Referencias

- [XXMI Launcher v2.2.1 fijado](https://github.com/SpectrumQT/XXMI-Launcher/tree/d56786b8dacb00c35204bff45ff5b8b83bd8962a)
- [Notas XXMI Launcher v2.1.5](https://github.com/SpectrumQT/XXMI-Launcher/releases/tag/v2.1.5)
- [XXMI Libraries v1.1.7 fijado](https://github.com/SpectrumQT/XXMI-Libs-Package/tree/6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9)
- [Investigación del loader](../../01_Descubrimiento/loader_xxmi.md)
- [Readiness ZZMI / Steam / Linux](../../01_Descubrimiento/launch_zzmi_steam_linux.md)
- [Verificación de LXMI 0.7](../../06_Verificacion/verificacion_0_7.md)
