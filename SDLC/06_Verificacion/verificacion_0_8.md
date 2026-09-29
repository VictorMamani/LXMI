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

Esta tabla compara evidencia concreta; el resultado Direct Inject no sustituye la paridad Hook requerida por el modo default de ZZMI.

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

## Validaciones de código

Registrar los resultados observados después de completar los comandos de cierre:

| Validación | Estado |
|---|---|
| `cargo fmt --all -- --check` | **PASS** |
| `cargo check --workspace` | **PASS** |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **PASS** |
| `cargo test --workspace` | **PASS**; 181 passed, 0 failed, 7 ignored (host/network/real-game opt-in). |
| `npm run typecheck` | **PASS** |
| `npm run lint` | **PASS** |
| `npm run format:check` | **PASS** |
| `npm run build` | **PASS** |
| `npm run tauri:dev` + revisión ventana nativa | **PASS**; scan, Loader Lab guard y readiness revisados en la ventana. |
| `git diff --check` | **PASS**; sin whitespace errors. |

## Decisión de avance

No iniciar LXMI 0.9. Antes se necesita una estrategia autorizada para verificar Hook con aislamiento de proceso demostrado, además de evidencias faltantes de Steam Linux/Proton, same-prefix y compatibilidad de ZZZ. No usar Direct Inject como sustituto silencioso del Hook default.
