# LXMI 0.8 — verificación Hook y ZZZ launch readiness

**Fecha:** 2026-09-28  
**Estado:** implementación parcial; Hook parity y gate ZZZ bloqueados por evidencia/alcance.

## Etiquetas

- **UPSTREAM VERIFIED:** fuente oficial fijada/revisada; no implica que LXMI reproduzca ese comportamiento.
- **COMPROBADO EN CÓDIGO:** implementación o test automatizado local.
- **FIXTURE:** datos/proceso sintético propio.
- **HOST TEST:** ejecución en Proton real del host.
- **NO COMPROBADO:** requiere una prueba que no se realizó.
- **BLOCKED:** no se ejecuta debido a un límite de seguridad explícito.

## Investigación Hook upstream

**UPSTREAM VERIFIED:** XXMI Launcher `v2.2.1`, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`; ZZMI package `v1.5.0`, commit `e59f87047cd405c5db5476d3b1b499574bc43d67`.

- `StartAndInject.use_hook` tiene default `true`; `MigotoInjector` configura `3dmloader.dll` para Hook.
- `run_hook_injector()` llama `HookLibrary(dll_path, HHOOK, mutex)` antes de iniciar el proceso configurado. El comentario upstream llama a esto setup de un hook global.
- El nombre del target se pasa a `WaitForInjection` y al process tracker; no se entrega a `HookLibrary` en la API Python inspeccionada.
- El launcher comprueba temprano, espera la ventana por process name, comprueba tardíamente y siempre llama `UnhookLibrary` en cleanup.
- ZZMI `d3dx.ini`: `target = ZenlessZoneZero.exe`, `loader = XXMI Launcher.exe`, `module = d3d11.dll`. El comentario de configuración define `loader` como identidad `.exe` permitida para carga de módulos desde ubicación externa/anidada. No es el target ni el nombre que LXMI nativo debe asumir.
- Inspección read-only del `3dmloader.dll` local: imports PE incluyen `SetWindowsHookExW` y `UnhookWindowsHookEx`. **No se ejecutó ni desensambló** en 0.8.
- Si el Hook DLL llega a mapearse en procesos distintos del target, cómo lo restringe el componente nativo y qué ámbito tiene dentro de Wine/Proton **UNKNOWN**. `d3dx.ini target` no demuestra que la instalación del hook esté limitada al proceso.

Fuentes y secuencia completa: [`loader_xxmi.md`](../01_Descubrimiento/loader_xxmi.md). Native/Direct Steam Launch: [`launch_zzmi_steam_linux.md`](../01_Descubrimiento/launch_zzmi_steam_linux.md).

## Hook Lab

| Caso | Estado | Resultado |
|---|---|---|
| Rechazo backend del modo Hook antes de filesystem/process work | **COMPROBADO EN CÓDIGO / FIXTURE** | `HookScopeNotIsolated`; la prueba usa rutas inexistentes y confirma que no intenta iniciar Proton. |
| Rechazo en el adaptador Tauri antes de Steam discovery/runtime resolution | **COMPROBADO EN CÓDIGO / HOST UI** | El servicio devuelve `hook_scope_not_isolated` en la primera rama. La prueba unitaria usa una ruta Proton inexistente; la ventana mostró que no se inspeccionó Proton ni se inició proceso. |
| Baseline con test target | **HOST TEST previo 0.7** | Pasa sin Hook. |
| Direct Inject positivo | **HOST TEST previo 0.7** | Pasa con DLL remota, marker y nonce en el fixture. No equivale a Hook. |
| Hook positivo bajo Proton | **BLOCKED / NO COMPROBADO** | No se ejecutó: el Hook global no cumple la regla target-only con evidencia actual. |
| Hook + DLL remota | **BLOCKED / NO COMPROBADO** | No se ejecutó. |
| Target/loader mismatch y DLL ausente en Hook | **NO COMPROBADO** | Ninguna variación del Hook se ejecuta; el guard bloquea todos los casos antes del proceso. |
| `loader = lxmi-hook-runner.exe` | **NO COMPROBADO** | No se alteró configuración ni se cambió la identidad upstream. |

La decisión ADR-027 sigue `NoDecision` para producción. No se presenta un fracaso funcional del Hook bajo Proton: se presenta un bloqueo previo a la prueba por falta de aislamiento demostrable.

### Comparación de capacidades observadas

| Capacidad | Direct Inject | Hook |
|---|---|---|
| Coincide con el modo default observado de ZZMI | No; ruta alternativa | Sí; `use_hook=true` en la configuración upstream inspeccionada |
| Ejecución bajo Proton | **HOST TEST previo 0.7: COMPROBADO** en prefix aislado | **BLOCKED**; no se ejecutó |
| DLL ubicada fuera del target | **HOST TEST previo 0.7: COMPROBADO** con marker/nonce | **NO COMPROBADO** |
| Orden respecto al target | El runner LXMI creó su target y dirigió Direct Inject a ese proceso | **UPSTREAM VERIFIED:** Hook setup ocurre antes de iniciar el target |
| Alcance limitado a un proceso | El executor 0.7 acepta solamente el target y PID que él mismo crea | **UNKNOWN**; el export `HookLibrary` no recibe target en la llamada inspeccionada |
| Espera/timeout | Acotado por el runner 0.7 | Upstream espera el target mediante `WaitForInjection`; el scope efectivo en Wine/Proton no se validó |
| Selección de modo en LXMI | Capability lab verificada, sin selección del juego | Backend y Tauri lo rechazan antes de discovery/Proton |

Esta tabla refleja el estado evaluado en LXMI 0.8. La investigación de 0.8.1 encontró que upstream expone también una ruta Direct Inject para Custom Launch; esa ruta no elimina la necesidad de demostrar compatibilidad específica con ZZMI/ZZZ en Linux/Proton.

## Readiness ZZZ

La nueva evaluación Tauri es read-only y muestra instalación/executable, compatdata/prefix, paquetes/autenticidad, runtime administrado, Proton candidates, selección del juego, Proton explícito del laboratorio, capabilities, requirements, blockers/unknowns y gate. El gate nunca ejecuta ZZZ.

Estados derivados por el servicio en este incremento:

- Hook en laboratorio: `Blocked`.
- Hook remote DLL: `Blocked`.
- Direct Inject fixture: `Verified` por el host test 0.7.
- Steam Native Launch: `UpstreamDocumented` para los ejemplos Windows.
- Direct Steam Launch de ZZZ: `Unsupported`/pendiente según la guía upstream consultada.
- ZZZ Steam en Linux/Proton: `Unverified`.
- Hook visible desde contexto Steam, same-prefix y compatibilidad específica del juego: `Unknown`/`NotTested`.
- Selección Proton para ZZZ: se conserva únicamente desde `GameRuntimePlan`; el runtime del Loader Lab no se copia.

**Readiness esperada:** `Blocked` o `NeedsEvidence`; nunca `ReadyForControlledExperiment` con los hechos actuales. No habilita 0.9.

## Validación local de ZZZ

- Ventana Tauri nativa recompilada: Scan Steam confirmó ZZZ Steam AppID `4162040`, directorio y `ZenlessZoneZero.exe` presentes; mostró `compatdata/4162040` y `pfx` existentes como candidato. **COMPROBADO**, solo discovery de lectura.
- La evaluación Tauri mostró un Proton candidato (`Proton Experimental`) y **Runtime seleccionado para el juego: Unknown**. El Proton elegido en el laboratorio 0.7 se muestra por separado. **COMPROBADO**.
- El snapshot de readiness Tauri mostró los paquetes ZZMI `1.50` y XXMI Libraries `1.1.7` administrados, autenticidad verificada por los metadatos existentes, pero decisión `blocked`. **COMPROBADO en esta sesión**, sin inferir compatibilidad de juego.
- No se ejecutó `ZenlessZoneZero.exe`, Steam, Proton/Wine para ZZZ ni ningún proceso de juego.
- No se modificó Steam, game files, launch options, `compatdata/4162040` ni su prefix.

## UI nativa

**COMPROBADO:** `npm run tauri:dev` abrió la aplicación desktop nativa; se ejecutó Scan Steam, se inspeccionó Loader Lab, se evaluó Launch Readiness y se accionó el botón Hook. La UI mostró el bloqueo del servicio: `No se inspeccionó Proton ni se inició ningún proceso.` El gate ZZZ mostró `blocked`; el runtime del juego quedó `Unknown`. Browser/Vite no se usó como sustituto de Tauri.

La primera interacción antes de corregir el orden del adaptador devolvió `runtime_not_found` porque se resolvía el Proton antes del guard. Se cambió el servicio para rechazar Hook como primera operación y se repitió el flujo en la ventana recompilada. No hubo lanzamiento Proton durante ninguna de estas comprobaciones.

## LXMI 0.8.1 — resolución de estrategia y reassessment

**Fecha de investigación:** 2026-09-29. Esta sección reevalúa el bloqueo de estrategia; no inicia Hook, Direct Inject, Proton ni ZZZ. La parte 0.8 anterior conserva su resultado histórico de readiness `Blocked`; el resultado vigente después de aplicar 0.8.1 se registra en esta sección.

### Trace upstream

**UPSTREAM VERIFIED:** XXMI Launcher `v2.2.1`, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`; ZZMI package `v1.5.0`, commit `e59f87047cd405c5db5476d3b1b499574bc43d67`.

- `ModelImporterPackage.__init__` inicializa `use_hook=true`; `ZZMIPackage` no sobrescribe el valor. `start_game()` propaga ese default en `StartAndInject`.
- Sin Custom Launch, upstream conserva el modo del paquete: ZZMI va a Hook.
- Con Custom Launch activo, `custom_launch_inject_mode == 'Hook'` selecciona Hook; `Inject` selecciona `use_hook=false` y el flujo Direct Inject genérico. `Bypass` también usa la rama `false`, pero no incluye `d3d11.dll` salvo selección explícita en Extra Libraries.
- Direct Inject lanza el comando configurado y luego busca el proceso por basename/PID con timeout; `3dmloader.dll` ejecuta su export `Inject` para la lista de DLLs (incluido `d3d11.dll` cuando corresponde). No se exige que el target esté vivo antes de empezar el comando, pero sí debe aparecer para que la inyección prospere.
- El mismo `MigotoInjector`/`3dmloader.dll` implementa la ruta genérica; las ramas de código reales llaman `run_hook_injector` para Hook y `run_direct_injector` para Inject. Dos comentarios adyacentes en upstream invierten los nombres de mecanismo; la conclusión usa las llamadas/cuerpos.
- `target`, `module` y `loader` de ZZMI siguen siendo configuración común del importer. `loader=XXMI Launcher.exe` no selecciona Hook; un proceso loader LXMI Windows con otra identidad no ha sido validado.

### WWMI precedent

**UPSTREAM VERIFIED:** las notas oficiales de XXMI Launcher v2.2.0 (release `6501b0c`, 2026-06-08) dicen que WWMI cambió el default a Direct Inject porque Hook dejó de funcionar. La rama upstream actual consultada muestra `WWMIPackage.self.use_hook=false`; la recuperación de su archivo al pin v2.2.1 no pudo verificarse desde el enlace fijado. Esto es precedente para WWMI, no evidencia de ZZMI. En el pin ZZMI v2.2.1 `use_hook` no aparece en `zzmi_package.py` y el valor sigue heredado `true`.

### Hook scope y prefix

El código Wine consultado (`wine-mirror/wine`, `server/hook.c`, `dlls/win32u/hook.c`, `server/wineserver.man.in`, consultado 2026-09-29; mirror `master`, commit no recuperado) almacena hooks globales en la tabla del desktop dentro del server actual. La documentación de `wineserver` indica que `WINEPREFIX` distinto permite sesiones Wine independientes. Esta evidencia general no valida la build Proton local, el mecanismo de `3dmloader.dll`, ni el requisito mismo prefix/wineserver de ZZZ.

| Afirmación | Estado |
|---|---|
| Hook en Wine general | `WineDesktopWithinServer`, **UPSTREAM DOCUMENTED**; no es proceso-bound. |
| Hook aislado bajo Proton | **UNKNOWN / NO COMPROBADO**. No se ejecutó; guard intacto. |
| Mismo prefix/wineserver requerido por ZZZ | **UNKNOWN**. No se inspeccionó/ejecutó el prefix para este trabajo. |
| Direct Inject 0.7 con fixture LXMI | **HOST TEST previo**, no repetido en este incremento. |
| Direct Inject compatible con ZZMI/ZZZ/Linux/Proton | **UNKNOWN / NO COMPROBADO**. |

### Decisión y readiness

- Respuesta de arquitectura upstream: **C — el mecanismo depende del modo de launch**. Hook es default ZZMI; Direct Inject existe en Custom Launch. Para compatibilidad ZZMI/ZZZ/Steam Linux/Proton la respuesta sigue **D — evidencia insuficiente**.
- `HookRequirementState = NotEstablished`; no se confunde con `HookNotRequired`.
- `DirectInjectCandidate = true` cuando upstream ofrece el modo y el fixture 0.7/verificación de DLL remota constan como evidencia. Criterios game-specific, loader identity en Linux, Steam/Proton, proceso/prefix y anti-cheat siguen Unknown/NotEstablished.
- `LoaderStrategyDecision.outcome = NoDecision`, `preferred_mode = None`.
- `GameRuntimePlan.selected` permanece `Unknown`; el runtime de laboratorio no se copia al juego.
- Readiness recalculado = `NeedsEvidence`; `ControlledGameExperimentGate.eligible_for_review = false`, `launches_game = false`.
- El guard backend `hook_scope_not_isolated` permanece en la primera rama de la operación Hook. No se ejecutó Hook ni Direct Inject en 0.8.1; no se inició Proton, Steam o ZZZ.

### Implementación y pruebas 0.8.1

Los tests agregados cubren selección no inferida, candidato vs. estrategia preferida, Hook `NotEstablished`, scope Wine no process-bound, documentación Steam vs. Linux, readiness/gate granulares y Proton de juego desconocido. Los datos son **FIXTURE**; la verificación Direct Inject bajo Proton corresponde al **HOST TEST 0.7**, no a una prueba repetida en 0.8.1.

| Validación | Resultado |
|---|---|
| `cargo fmt --all -- --check` | **PASS** |
| `cargo check --workspace` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **PASS** |
| `cargo test --workspace` | **PASS:** 186 passed, 0 failed, 7 ignored (opt-in/host/network) |
| `npm run typecheck` | **PASS** |
| `npm run lint` | **PASS** |
| `npm run format:check` | **PASS** |
| `npm run build` | **PASS** |
| `npm run tauri:dev` + revisión nativa read-only | **COMPROBADO:** ventana Tauri; scan Steam; ZZMI 1.5.0 y XXMI Libraries 1.1.7 seleccionados desde storage; readiness `NeedsEvidence`; Proton de juego `Unknown`; no se accionó Loader Lab |
| `git diff --check` | **PASS** |

La inspección nativa de paquetes encontró tres inventarios administrados: ZZMI v1.5.0, ZZMI v1.4.5 y XXMI Libraries v1.1.7. La evaluación con los paquetes por defecto seleccionados mostró instalación/executable y prefix candidato presentes, runtime ensamblado y autenticidad verificados, estrategia `NoDecision`, Hook `Blocked`, Direct Inject `Verified` únicamente para el fixture LXMI, compatibilidad ZZMI/ZZZ `No probado`, y gate `needs_evidence` no elegible. El botón de readiness solo ejecutó discovery/lectura de metadatos; no se inició Steam, Proton, Wine, Hook, Direct Inject ni ZZZ.

La ventana se cerró al terminar; el servidor de desarrollo en `127.0.0.1:1420` dejó de escuchar. No hubo modificaciones de Steam, archivos del juego, `compatdata/4162040` ni prefix.

El detalle y los enlaces upstream están en [`estrategia_loader_zzmi_linux.md`](../01_Descubrimiento/estrategia_loader_zzmi_linux.md). El panel de readiness no ofrece acciones de ejecución.

## Decisión de avance

La evidencia upstream permite responder **C** sobre las rutas disponibles por modo de launch: Hook es el default de ZZMI y existe Direct Inject en Custom Launch. Para ZZMI + ZZZ + Steam Linux/Proton, la decisión es **D: evidencia insuficiente**. El estado LXMI es `NoDecision` y `NeedsEvidence`; el gate permanece cerrado. No iniciar LXMI 0.9 ni ejecutar Hook, Direct Inject, Proton, Steam o ZZZ hasta que el trabajo de evidencia siguiente resuelva los requisitos correspondientes. No inferir la selección de Proton del juego ni compatibilidad a partir del Loader Lab.
