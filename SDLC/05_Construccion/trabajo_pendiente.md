# Trabajo pendiente

## LXMI-0.1 a 0.5.3 — estado

- [x] Workspace Tauri 2/React/TypeScript/Rust con `lxmi-core`, `lxmi-steam`, `lxmi-proton` y `lxmi-runtime`.
- [x] Detección Steam roots y bibliotecas, manifests, Wuthering Waves, compatdata/pfx y compatibility tools en modo de solo lectura.
- [x] Reutilizar Valve KeyValues; reconocer Proton y distinguir Steam Linux Runtime mediante metadata estructural.
- [x] Fixtures para VDF/ACF, Proton, custom compatibility metadata, SLR, estado incompleto, rutas y symlinks.
- [x] Planner efímero por juego con selección desconocida, candidates separados, requisitos, evidence, issues y readiness.
- [x] Test explícito: un único Proton candidato no implica selección.
- [x] `cargo fmt --all -- --check`, check, Clippy con `-D warnings`, tests Rust de workspace (69), frontend typecheck/lint/format/build.
- [x] Escaneo real de Steam: Wuthering Waves con directorio presente, compatdata/pfx ausentes, Proton Experimental + Hotfix, tres Steam Linux Runtime; selección desconocida y `NeedsInitialization`.
- [x] El symlink irrelevante `Steam.dll` se omite sin seguirlo, se clasifica informativo y no marca parcial el discovery.
- [x] Arranque de ventana Tauri comprobado con `npm run tauri:dev`.
- [x] El repositorio está en la ruta LXMI, branch `main`, con remote `origin` `git@github.com:VictorMamani/LXMI.git`.
- [ ] Verificar una herramienta custom instalada localmente; su formato y rutas están cubiertos por fixtures.
- [x] Revisar upstream XXMI Launcher, WWMI-Package y XXMI-Libs-Package, layouts de releases y licencias consultables. Componentes redistribuibles no identificados permanecen bloqueados para empaquetado.
- [x] Añadir `lxmi-xxmi`: modelos separados para WWMI y XXMI Libraries, inspección estructural limitada y evidence; no acredita autenticidad/compatibilidad.
- [x] Importar carpetas a `$XDG_DATA_HOME/lxmi` con fallback ya resuelto por `SystemInfo`; staging privado, SHA-256, manifest fuera del payload y promoción sin overwrite.
- [x] Crear assessment y plan declarativo con requirements, candidatos y hashes/backup esperado; sin executor/apply ni mutaciones fuera del storage LXMI.
- [x] Exponer estado, import local y revisión de plan en la UI Tauri; no rediseñar ni integrar el prototipo visual.
- [x] Comprobar ambos assets upstream en `/tmp` sin ejecutar contenido; validarlos/importarlos con el ejemplo aislado.
- [x] 45 tests nuevos de estructura, límites, hashes, integridad, storage, symlinks, assessment, plan y serialización IPC; workspace 114 tests.
- [x] `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy `-D warnings`, `cargo test --workspace`, typecheck, lint, format, build y arranque Tauri v0.5.
- [x] Scan local read-only: juego directory presente; ejecutable Win64 esperado, compatdata y prefix ausentes; selección desconocida, readiness `NeedsInitialization`; no se detecta runtime en ubicaciones enumeradas. Esto no descarta launcher instalado externamente.
- [x] Nueva observación local read-only (2026-09-28): Wuthering Waves no está en los manifests de la biblioteca detectada; las carpetas Proton Experimental/Hotfix observadas carecen de metadata/entrypoint y no califican como tools; compatdata ausente; plan `Blocked`. Ver `06_Verificacion/verificacion_0_5.md`.
- [x] Registrar Zenless Zone Zero en el catálogo con AppID Steam `4162040`; conservar Wuthering Waves y sus fixtures.
- [x] Descubrir ejecutables ZZZ mediante scan acotado; distinguir manifest, directorio, ejecutable, compatdata y prefix.
- [x] Añadir ZZMI como integración asociada a ZZZ; rechazar paquetes cruzados WWMI/ZZMI y modelar XXMI Libraries por separado.
- [x] Contrastar estructura, ejecutables, versión raw, dependencia, releases, metadata de firma y licencias upstream; registrar incógnitas de Steam/Linux/Proton en `01_Descubrimiento/ecosistema_xxmi_zzmi.md`.
- [x] Mantener import mediante el mismo pipeline de staging/storage gestionado; no descargar ni importar una release real en 0.5.1.
- [x] Generar plan ZZMI declarativo, no ejecutable y sin destinos del juego/prefix.
- [x] Añadir tests sintéticos ZZZ/ZZMI y prueba host-only ignorada para el scan real; conservar selección de Proton desconocida.
- [x] Scan local read-only ZZZ: Steam/AppID 4162040, directorio/exe presentes, `compatdata/4162040/pfx` candidato; un Proton candidate, selection `unknown`, readiness `incomplete`.
- [x] Tauri v0.5.1 compila e inicia. Scan ejecutado por el snapshot del command; click manual en la ventana queda **NO COMPROBADO**.
- [x] Consultar releases oficiales fijadas y usar la selección vigente que mostró LXMI: ZZMI v1.5.0 y XXMI Libraries v1.1.7; descargar ambos assets oficialmente, verificar SHA-256 y firmas upstream, extraer de forma segura e importar a storage administrado. La versión ZZMI v1.4.5 consultada inicialmente permanece como paquete histórico. Ver `06_Verificacion/verificacion_0_5_2.md`.
- [x] Construir package inventories reales y resolver la dependencia ZZMI + XXMI Libraries sin fusionar los paquetes.
- [x] Generar mapping relativo y ejecutar dry-run hash-only contra la carpeta candidata derivada del ejecutable ZZZ. La raíz configurada `importer_path` sigue desconocida; no se aprueba apply.
- [x] Completar interacción manual nativa: scan Steam/ZZZ, consulta de releases, descarga/verificación/import de ZZMI y XXMI Libraries, confirmación de dependencia y revisión del dry-run de 49 mappings; UI reportó `apply` y escrituras deshabilitados.
- [x] Fijar la topología upstream de `App.Root`/`ZZMI/`, desplazar el runtime a storage privado LXMI y reclasificar el dry-run 0.5.2 como comparación histórica, no target autorizado.
- [x] Crear un plan de ensamblado y materializar ZZMI + las DLL que upstream ubica en `importer_path`, preservando paquetes fuente, autenticidad e inventarios; derivar `d3dx.ini` sin mutar upstream y mantener `Mods/` bajo LXMI.
- [x] Inspeccionar `pfx/dosdevices` mediante lectura directa y mapear rutas Linux a Windows sin crear o alterar mappings; mantener selección Proton, estrategia loader y requisito de mismo prefix como `Unknown`.
- [x] Añadir `LaunchTopologyPlan`, comandos Tauri y revisión/ensamblado separados; no hay executor, lanzamiento, inyección ni escrituras en Steam, ZZZ, compatdata o prefix.
- [x] Validación host-only opt-in con paquetes oficiales ZZMI v1.5.0 y XXMI Libraries v1.1.7: runtime privado de 49 archivos ensamblado; mapping del importer por `Z:`; selección Proton desconocida; snapshots de rutas candidatas del juego y `dosdevices` idénticos antes/después.
- [x] Fijar upstream XXMI Launcher v2.2.1 por tag/commit y registrar límites de `3dmloader.dll`/`3dmloader.exe`, launcher portable y licencias por componente. La evidencia no basta para elegir helper ni afirmar compatibilidad ZZZ + Steam + Linux/Proton.

## LXMI-0.6 — Controlled Windows Runtime Bridge Prototype

**CERRADO (2026-09-28).** El helper Windows quedó enlazado con MinGW, staged y validado por hash. Pasaron el host bridge positivo y negativo con Proton Experimental explícitamente seleccionado, prefix aislado, mapping real comprobado por el helper y la interacción manual en la ventana Tauri. No iniciar LXMI 0.7 en este incremento.

- [x] Añadir crates de dominio/protocolo y helper Windows inocuo.
- [x] Implementar stdin/stdout JSON v1, nonce, validación del runtime marker y límites de entrada/salida/tiempo.
- [x] Mantener la selección bridge separada de la selección de runtime del juego.
- [x] Restringir el helper al storage administrado de LXMI y al prefix aislado; rechazar ejecución usando compatdata de ZZZ.
- [x] Ejecutar quality gates Rust y frontend; iniciar `npm run tauri:dev` y probar en la ventana Tauri nativa.
- [x] Compilar helper `x86_64-pc-windows-gnu` con linker MinGW explícito. El intento anterior con Zig 0.15.2 falló por `msvcrt`; quedó resuelto, sin alterar linker Linux ni instalar paquetes con sudo.
- [x] Generar PE32+ de 1,517,755 bytes y SHA-256 `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2`.
- [x] Ejecutar `bash tools/install-bridge-helper.sh`; el helper staged y su sidecar/hash coinciden bajo `$XDG_DATA_HOME/lxmi/helpers/`.
- [x] Elegir Proton Experimental explícitamente como Bridge test runtime y aceptar la advertencia. La selección Proton del juego se mantuvo `unknown`.
- [x] Host test positivo: protocolo v1, nonce roundtrip, helper exit 0, path `Z:\...` realmente leído y SHA-256 del manifiesto coincidente.
- [x] Host test negativo: path gestionado inexistente devuelve JSON `runtime_not_visible`, nonce correlacionado y exit 2; sin panic ni timeout.
- [x] Validar en UI nativa éxito, helper ausente (restaurado después) y runtime ZZMI no disponible al no seleccionar packages.
- [x] Registrar antes/después: ZZZ executable/targets vigilados y `compatdata/4162040/pfx` no se modificaron; Proton inicializó solo el prefix LXMI aislado y actualizó `dist.lock` de su propia distribución.

## LXMI-0.7 — Upstream Loader Compatibility Experiment

**CERRADO (2026-09-28).** El experimento usó exclusivamente el test target propiedad de LXMI con un Proton explícito y prefix aislado. El export Direct Inject de `3dmloader.dll` cargó el test DLL remoto y produjo marker/nonce válidos. Se recorrieron baseline, positivo y target ausente en Tauri nativo. No se interactuó con ZZZ ni su compatdata.

- [x] Fijar evidencia de XXMI Launcher v2.2.1 y contrastar las notas v2.1.5 sobre `3dmloader.exe`.
- [x] Determinar que el modo default observado de ZZMI es Hook mediante `3dmloader.dll`; el test Direct Inject es una ruta alternativa y no equivale al modo default.
- [x] Verificar release/firma/hash de `3dmloader.dll` local; licencia/capacidad registrada como uso ejecutable local únicamente. No redistribuir.
- [x] Compilar runner, target y test DLL Windows x86-64 con MinGW; staging únicamente dentro del storage LXMI.
- [x] Crear `LoaderExperimentPlan` tipado, paths cerrados, nonce y validación de target. No existe API genérica de PID/DLL/ejecutable.
- [x] Proton host test aislado: baseline, Direct Inject positivo, target ausente, DLL ausente y nonce incorrecto. Marker, PID/identidad y mapping remoto fueron comprobados.
- [x] Interacción manual desde la ventana Tauri nativa: inspección, selección explícita de Proton, aceptación de la advertencia, baseline, Direct Inject y rechazo target ausente.
- [x] Registrar que no se usó ZZZ, Steam, HoYoPlay, `compatdata/4162040`, prefix del juego ni anti-cheat.
- [x] Actualizar investigación, ADR, arquitectura, seguridad, README, registro y verificación.

## LXMI-0.8 — Hook Mode Parity & ZZZ Launch Readiness

**Cerrado con bloqueo de seguridad (2026-09-28).** El Hook upstream se prepara antes del target y el export `HookLibrary` no recibe nombre de target; el guard fail-closed del Loader Lab impidió ejecutarlo sin evidencia de aislamiento target-only. La estrategia de producción quedó `NoDecision`; el runtime seleccionado por Steam para ZZZ se conservó como `Unknown`.

- [x] Fijar la ruta Hook ZZMI upstream a XXMI Launcher v2.2.1 y explicar setup, inicio, verificación temprana/tardía y cleanup.
- [x] Separar `loader=` (identidad permitida para carga externa del módulo) de `target=` y del nombre de proceso pasado a `WaitForInjection`.
- [x] Añadir rechazo Hook fail-closed en backend y servicio Tauri antes de filesystem/process work.
- [x] Crear `ZzmiLaunchReadiness` con capacidades, requirements, unknowns y gate declarativo sin executor.
- [x] Mostrar Hook bloqueado y readiness en Advanced, sin acción de lanzamiento.
- [x] Confirmar desde Tauri nativo ZZZ Steam `4162040`, executable, compatdata/pfx y Proton candidato; selección de Proton del juego `Unknown`.

## LXMI-0.8.1 — Loader Strategy Resolution for ZZMI on Linux

**Resultado de investigación/modelado (2026-09-29): `NoDecision`; readiness `NeedsEvidence`.** Hook sigue bloqueado en el laboratorio. Direct Inject permanece como candidato de investigación basado en ruta genérica upstream y el host test fixture de 0.7; no se declara compatible con ZZMI/ZZZ/Linux/Proton. No se ejecutaron Hook, Direct Inject, Proton, Steam ni ZZZ.

- [x] Fijar XXMI Launcher `v2.2.1` (`d56786b8dacb00c35204bff45ff5b8b83bd8962a`) y trazar `use_hook`/`custom_launch_inject_mode`.
- [x] Distinguir default Hook de ZZMI y la ruta Inject genérica de Custom Launch; documentar `Bypass` sin inferir soporte de ZZZ.
- [x] Comparar ZZMI con el precedente de WWMI y marcar claramente que ese precedente no prueba ZZMI.
- [x] Registrar `HookRequirement = NotEstablished`, sin tratarlo como `HookNotRequired`.
- [x] Revisar el scope global de Hook en Wine y mantener desconocidos el aislamiento bajo Proton y los requisitos same-prefix/same-wineserver.
- [x] Añadir `LoaderStrategyDecision`, evidencia, scope de aislamiento, criterios de candidatura y componentes granulares del gate.
- [x] Mantener el Proton del juego `Unknown`, el Hook guard activo y el gate cerrado.
- [x] Resumir la decisión en Advanced → ZZZ Readiness; conservar los diagnósticos en secciones expandibles.
- [x] Ejecutar las quality gates 0.8.1 y completar la revisión nativa read-only de scan, paquetes seleccionados y ZZZ readiness; el panel mostró `NeedsEvidence`, estrategia `NoDecision`, Hook bloqueado, Direct Inject verificado solo en fixture y Proton del juego `Unknown`.

## LXMI-0.8.2 — Steam / Proton Process Context Evidence

**Completado (2026-09-30):** se añadió inspección read-only y acotada de `config/config.vdf`, modelo separado para mapping por AppID/default global/tools instaladas frente a runtime efectivo, y exposición en la vista ZZZ Readiness. En el host, el bloque `CompatToolMapping` está vacío; `compatdata/4162040` y `pfx` existen como candidatos, pero no prueban ejecución efectiva. Proton efectivo, prefix efectivo y relaciones con wineserver/loader permanecen `Unknown`; readiness `NeedsEvidence`, gate no elegible y loader `NoDecision`.

- [x] Inspeccionar fuentes locales de configuración Steam y documentar su semántica con etiquetas de evidencia; no inferir runtime de tools instaladas.
- [x] Reutilizar Valve KeyValues y añadir fixtures para mapping por AppID, global, ausente e inválido.
- [x] Modelar fuentes/findings de configuración, manifest, biblioteca, compatdata, prefix candidato y contexto efectivo desconocido.
- [x] Integrar evidence en readiness/gate sin executor ni cambios en `LoaderStrategyDecision`.
- [x] Mostrar mapping AppID/global, herramientas con origen/path y relaciones desconocidas en Tauri.
- [x] Quality gates Rust y frontend pasan; `git diff --check` se registra en `06_Verificacion/verificacion_0_8_2.md`.
- [x] Confirmar localmente manifest `4162040`, ejecutable, `compatdata/4162040/pfx`, Proton Experimental candidato y ausencia de mapping AppID/global en el bloque inspeccionado; no ejecutar Steam/Proton/Wine/ZZZ.

Se preservan los cambios sin commit de 0.8.1 y se añadieron cambios de 0.8.2 en la misma rama. No se hizo commit ni push. Las incógnitas de contexto de proceso siguen abiertas; este resultado no autoriza un experimento de lanzamiento.

## LXMI-0.8.3 — Passive ZZZ Runtime Observation

**CERRADO (2026-09-30, hora local).** La ventana Tauri nativa mostró un proceso ZZZ observado, AppID `4162040`, Proton Experimental, compatdata y prefix. La versión de aplicación/proyecto es `0.8.3`. Esta evidencia es por sesión: no cambia la selección configurada ni autoriza loaders/lanzamiento.

- [x] Preservar el árbol existente de 0.8.1/0.8.2; trabajar en `main` sin commit/push.
- [x] Añadir domain observer con filtrado por usuario, allowlist exacta de environment, PID starttime revalidado, process ancestry limitada, ambigüedad y estados de permiso/desaparición.
- [x] Instrumentar scans con contadores agregados y razones de descarte; distinguir `PermissionDenied` de `Unavailable` sin loguear command lines ni environment completo.
- [x] Representar `active`, elapsed y estados terminales `Completed`, `ProcessNotFound`, `TimedOut`, `ProcessExited`, `AmbiguousProcess` y `BackendError` en el snapshot serializado.
- [x] Probar con fixture que un ZZZ ya presente antes de comenzar se detecta en el primer scan, incluyendo `/proc/exe` de Wine/preloader y evidencia de AppID/compatdata en ascendencia.
- [x] Mantener configurado, disponible y observado como evidencia separada; no modificar `GameRuntimePlan.selected`, `LoaderStrategyDecision` o la política de gate.
- [x] Exponer observación consentida en Advanced, conservar estado efímero con redacción del home y mantener resultado/errores visibles incluso con el disclosure plegado.
- [x] Ejecutar `cargo fmt`, `cargo check`, `cargo clippy -D warnings` y `cargo test --workspace`.
- [x] Ejecutar frontend typecheck/lint/format/build y `git diff --check`; pasan.
- [x] Ejecutar `npm run tauri:dev`; Vite y el proceso nativo `lxmi-desktop` iniciaron.
- [x] Validar en Tauri nativo: el resultado visible fue `Observado · En curso`; PID `691954`, `ZenlessZoneZero.exe`, Steam AppID `4162040`.
- [x] Registrar Proton de esta ejecución: `Proton - Experimental`, internal ID `1790244902`, build `experimental-11.0-20260924-x86_64`; config sin mapping por AppID en la fuente inspeccionada y default global separado.
- [x] Registrar compatdata `~/.local/share/Steam/steamapps/compatdata/4162040` como observado/coincidente y prefix `.../pfx/` como observado.
- [x] Registrar ascendencia observada y evidencia de proceso wineserver del UID actual con `WINEPREFIX` coincidente; esto no prueba que el juego use esa instancia.
- [x] Registrar la limitación allowlist: `environ` fue denegado en el contexto del juego/ascendencia; el contador directo de candidatos marcó cero. Se añadió UI/backend para exponer esos alcances por separado.
- [x] Mantener `GameRuntimePlan.selected = Unknown`, loader strategy `NoDecision`, readiness `NeedsEvidence` y gate cerrado.
- [x] Verificar el flujo del observer por código/tests: procfs read-only, sin señales, sin lectura de memoria y sin launch/injection; no se tomó un hash baseline pre/post del árbol completo del juego.
- [x] Actualizar la aplicación a `0.8.3` tras la validación manual y documentar findings/unknowns.
- [x] Ejecutar los quality gates después de las últimas modificaciones; resultados en `SDLC/06_Verificacion/verificacion_0_8_3.md`.

## Estado de Git

- LXMI se trabaja en branch `main` en el repositorio independiente `04_Proyectos/19_LXMI`, con `origin` `git@github.com:VictorMamani/LXMI.git`.
- Los cambios previos 0.8.1/0.8.2 y el incremento 0.8.3 continúan sin commit ni push; preservar el árbol completo al preparar un checkpoint.

## Antes de integrar runtimes o contenido de mods

- [x] Registrar repositorios upstream oficiales consultados de XXMI/WWMI y licencias publicadas; revisar individualmente todo binario/recurso redistribuible antes de distribución.
- [ ] Verificar documentación y políticas aplicables del juego/runtime; no evadir anti-cheat ni controles.
- [x] Diseñar importación segura de directorio y ZIP; probar traversal, rutas absolutas, enlaces, colisiones, profundidad, tamaño y relación de compresión.
- [ ] Investigar función live e IPC de forma separada; no asumir que existe un protocolo upstream.
- [ ] Evaluar SteamOS y distribución solo después de un MVP probado.
