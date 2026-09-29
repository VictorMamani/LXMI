# LXMI 0.7 — investigación del loader upstream

**Consultado y probado:** 2026-09-28. Los enlaces a código se fijan a commits; las release notes se vinculan al tag.

## Fuentes fijadas

| Repositorio | Referencia | Archivos/evidencia |
|---|---|---|
| `SpectrumQT/XXMI-Launcher` | tag `v2.2.1`, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a` | [`zzmi_package.py`](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/model_importers/zzmi_package.py), [`migoto_package.py`](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/migoto_package.py), [`dll_injector.py`](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/utils/dll_injector.py) |
| `SpectrumQT/XXMI-Launcher` | tag `v2.1.5`, release commit corto `abc8087` mostrado por GitHub | [notas oficiales de v2.1.5](https://github.com/SpectrumQT/XXMI-Launcher/releases/tag/v2.1.5) |
| `SpectrumQT/XXMI-Libs-Package` | tag `v1.1.7`, commit `6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9`, release id `387957029` | [source tree](https://github.com/SpectrumQT/XXMI-Libs-Package/tree/6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9), [`COPYING.txt`](https://github.com/SpectrumQT/XXMI-Libs-Package/blob/6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9/COPYING.txt) |

Las releases consultadas y el asset/component signature se revalidaron mediante el package store de LXMI. El test no descarga desde GitHub ni modifica el package original; toma el `3dmloader.dll` cuyo hash está registrado en ese package verificado.

## Ruta upstream relevante para ZZMI

En el código fijado, `ZZMIConfig` hereda `use_hook = true` de `StartAndInject`. El `MigotoInjector` pasa `3dmloader.dll` a `DllInjector`; con Hook llama a `HookLibrary`/`UnhookLibrary` y al flujo que instala el hook global antes de iniciar el proceso. La alternativa con `use_hook = false` recorre el camino directo y llama al export `Inject` con el PID del proceso objetivo y las DLL declaradas. Los comentarios adyacentes a las ramas en `migoto_package.py` describen esos métodos al revés respecto de las funciones llamadas; la conclusión aquí se deriva de las llamadas y del código de `dll_injector.py`, no de esos comentarios.

**Resultado para ZZMI:** la ruta upstream por defecto observada es **Hook** a través de `3dmloader.dll`; no se observó que ZZMI elija `3dmloader.exe` como su default. En v2.1.5 las notas indican que se sustituyó `pyinjector` por un injector propio en `3dmloader.exe` para el modo Direct Inject basado en `WriteProcessMemory` (default de EFMI según esas notas). Esto es evidencia de una implementación/versión concreta, no prueba de que toda integración ZZMI use ese ejecutable. La revisión del código v2.2.1 muestra a `DllInjector` cargando `3dmloader.dll` y resolviendo su export `Inject`; no se ejecutó `3dmloader.exe` en LXMI.

El experimento 0.7 eligió **Direct Inject** exclusivamente para una prueba controlada, invocando `Inject` del componente upstream `3dmloader.dll` contra el PID devuelto por el `CreateProcessW` que creó el único `lxmi-loader-test-target.exe`. No enumeró procesos ni aceptó PID, nombre, target o DLL desde React. El modo Hook se excluyó porque su instalación es global y excede el alcance de una prueba aislada.

Esto **no es el modo predeterminado ZZMI**, no prueba su configuración `[Loader]`, ni establece compatibilidad de ZZMI/ZZZ/Steam/Linux/Proton.

## Componentes y licencia

- El package XXMI Libraries v1.1.7 está separado del package ZZMI y aporta `3dmloader.dll`; LXMI verificó la firma upstream del componente y el hash antes de copiarlo al storage privado `tests/loader-v1/`.
- El repositorio de XXMI Libraries declara GPLv3 en `COPYING.txt`. La revisión del binario permite este uso de prueba local dentro del entorno del usuario; LXMI **no redistribuye** `3dmloader.dll` en Git ni en sus artefactos. La categoría interna queda `VerifiedExecutableOnly`; no equivale a autorización de redistribución.
- La release v2.1.5 y su nota sobre `3dmloader.exe` no se usaron como procedencia del componente probado. No se copió ni ejecutó `3dmloader.exe` o el launcher portable.
- No se copió código fuente upstream. La DLL y la integración de prueba permanecen fuera del repositorio, bajo `$XDG_DATA_HOME/lxmi/tests/loader-v1/`.

## Capacidades, alcance y límites

| Capacidad | Resultado de 0.7 |
|---|---|
| Ejecutar un helper/test target Windows bajo el Proton de prueba explícito | **HOST TEST — comprobado** |
| Cargar el test DLL externo con el export Direct Inject de `3dmloader.dll` | **HOST TEST — comprobado** para el target fijo LXMI |
| Mapping Linux→Windows de target y DLL remotos | **HOST TEST — comprobado** por mapping del prefix y paths devueltos por el marker |
| Rechazar un target fijo ausente sin fallback | **HOST TEST — comprobado**, exit `10` |
| Rechazar DLL ausente y nonce incorrecto | **HOST TEST — comprobado**, exits `11` y `12` |
| Modo Hook global | **No probado / fuera del experimento** |
| `3dmloader.exe` | **No probado** |
| Default Hook de ZZMI, config de `d3dx.ini` o identity `XXMI Launcher.exe` | **No probado** |
| Lanzamiento/carga en `ZenlessZoneZero.exe` o interacción anti-cheat | **No probado y prohibido en 0.7** |
| Compatibilidad ZZMI + ZZZ Steam + Linux/Proton | **Desconocida** |

El test usa el prefix aislado de LXMI `test-prefixes/loader-v1/compatdata`; no usa `compatdata/4162040`. La relación de prefix requerida por ZZZ sigue **UNKNOWN**. No se cambia Steam, juego, prefix de juego ni selección de Proton del planner.

## Diagrama del laboratorio

```mermaid
flowchart LR
    LXMI[LXMI nativo] -->|runtime de prueba explícito| Proton[Proton Experimental]
    Proton --> Runner[Runner LXMI de rutas fijas]
    Runner -->|CreateProcessW · PID propio| Target[lxmi-loader-test-target.exe]
    Runner -->|3dmloader.dll Inject · PID del hijo| Target
    Runner -->|ruta externa| TestDLL[lxmi-loader-test.dll]
    Target -->|DLL verifica proceso + nonce| Marker[Marker JSON controlado]
    LXMI -->|valida proceso · marker · nonce · paths| Result[Resultado tipado]
    ZZZ[ZenlessZoneZero.exe] -. no se usa .-> Runner
    Hook[Modo Hook global] -. excluido .-> Target
```

## Fuentes

- [Release XXMI Launcher v2.1.5](https://github.com/SpectrumQT/XXMI-Launcher/releases/tag/v2.1.5)
- [ZZMI package, launcher v2.2.1 fijado](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/model_importers/zzmi_package.py)
- [Migoto orchestration, launcher v2.2.1 fijado](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/migoto_package.py)
- [DllInjector API/orchestration, launcher v2.2.1 fijado](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/utils/dll_injector.py)
- [XXMI Libraries v1.1.7 fijado](https://github.com/SpectrumQT/XXMI-Libs-Package/tree/6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9)
- [Verificación y resultados del host LXMI 0.7](../06_Verificacion/verificacion_0_7.md)

## LXMI 0.8 — Hook mode y límites del laboratorio

**Consulta upstream:** 2026-09-28. Código fijado a XXMI Launcher `v2.2.1`, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`. ZZMI package inspeccionado localmente como release `v1.5.0`, commit `e59f87047cd405c5db5476d3b1b499574bc43d67`. Su `d3dx.ini` declara `target = ZenlessZoneZero.exe`, `loader = XXMI Launcher.exe` y `module = d3d11.dll`.

### Secuencia observada en upstream

`StartAndInject.use_hook` tiene valor predeterminado `true`. Para esa ruta, `MigotoInjector.run()` configura `DllInjector` con `load_hook=true`, carga `3dmloader.dll` y ejecuta `run_hook_injector()`.

1. El proceso controlador de XXMI Launcher carga `3dmloader.dll` y llama `HookLibrary(dll_path, hook_handle, mutex)`. El comentario del código upstream lo describe como instalación de un hook Windows global.
2. Después, `open_process()` inicia el `start_exe_path` configurado (o espera inicio manual). En Native Steam Launch para ZZZ, la guía configura `steam.exe -applaunch 4162040`; Steam es quien termina iniciando el juego.
3. El launcher consulta `WaitForInjection(dll_path, target_process, 5)` para detección temprana, espera la ventana por nombre de proceso con el process tracker y vuelve a consultar `WaitForInjection` para verificación tardía.
4. El cleanup llama `UnhookLibrary` y descarga el DLL del injector.

```mermaid
sequenceDiagram
    participant X as XXMI Launcher
    participant L as 3dmloader.dll
    participant S as Start process / Steam
    participant G as Target process
    X->>L: HookLibrary(dll_path, hook, mutex)
    Note over X,L: Hook preparado antes de iniciar el proceso
    X->>S: open_process(start_exe_path, args)
    S->>G: iniciar el proceso configurado
    X->>L: WaitForInjection(dll_path, target_process, timeout)
    X->>G: esperar ventana por nombre de proceso
    X->>L: WaitForInjection tardío
    X->>L: UnhookLibrary()
```

El código Python no entrega el nombre del target a `HookLibrary`; lo entrega a `WaitForInjection` y al process tracker. Así, `[Loader]target` identifica el proceso para el runtime 3Dmigoto, pero la evidencia consultada no demuestra que limite el ámbito de instalación del hook antes de que el módulo procese su configuración. El `3dmloader.dll` local autenticado importa `SetWindowsHookExW` y `UnhookWindowsHookEx`; la inspección no ejecutó ni desensambló ese mecanismo. El alcance de este hook bajo Wine/Proton —incluidos otros procesos/prefixes visibles— queda **UNKNOWN**.

### Semántica de `[Loader]loader`

El comentario en el `d3dx.ini` de ZZMI describe `loader` como el ejecutable al que se permite cargar el DLL de 3Dmigoto desde ubicaciones externas o anidadas. `target` es el proceso de juego al que se destina el módulo. No son sinónimos: `loader = XXMI Launcher.exe` no ordena a LXMI nativo renombrarse y tampoco es el parámetro del export `HookLibrary`. La identidad alternativa `lxmi-hook-runner.exe` no se probó y no se afirma que sea válida.

### Hook Lab de LXMI

El laboratorio 0.8 conserva paths fijos, nonce, runtime explícito y fixture de 0.7, pero rechaza `Hook` en backend con `HookScopeNotIsolated` antes de validar rutas, iniciar Proton o invocar binarios. Upstream prepara el hook global antes de arrancar el target; ni `[Loader]target` ni el resultado de `WaitForInjection` prueban que la carga inicial quede limitada al target. Ejecutar el positivo violaría la política del laboratorio de afectar solamente procesos de test propiedad de LXMI.

- **COMPROBADO EN CÓDIGO:** Hook se prepara antes del inicio del target; se usa `3dmloader.dll`; upstream espera/verifica el target y retira el hook en cleanup.
- **UPSTREAM VERIFIED:** ZZMI hereda `use_hook=true` en XXMI Launcher v2.2.1; `loader` es la identidad ejecutable permitida por la configuración de 3Dmigoto.
- **HOST TEST 0.7:** Direct Inject carga el fixture bajo Proton; no es paridad Hook.
- **BLOCKED / NO COMPROBADO COMO EJECUCIÓN:** positivo Hook, DLL remota vía Hook, mismatches de target/loader y funcionamiento Hook bajo Proton.
- **UNKNOWN:** filtro interno anterior a mapear el DLL, visibilidad entre procesos Wine/prefixes y requisito same-prefix.

No se implementa un hook propio, no se buscan procesos arbitrarios, no se usa ZZZ/Steam y no se interactúa con anti-cheat.

### Fuentes fijadas

- [`migoto_package.py` en v2.2.1](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/migoto_package.py)
- [`dll_injector.py` en v2.2.1](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/utils/dll_injector.py)
- [`zzmi_package.py` en v2.2.1](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/model_importers/zzmi_package.py)
- [Release XXMI Launcher v2.2.1](https://github.com/SpectrumQT/XXMI-Launcher/releases/tag/v2.2.1)
- [ZZMI `d3dx.ini` en release v1.5.0 fijada](https://github.com/leotorrez/ZZMI-Package/blob/e59f87047cd405c5db5476d3b1b499574bc43d67/ZZMI/d3dx.ini)
- [Wiki upstream: Native Steam Launch y Direct Steam Launch](https://github.com/SpectrumQT/XXMI-Launcher/wiki/More-Start-Options)
