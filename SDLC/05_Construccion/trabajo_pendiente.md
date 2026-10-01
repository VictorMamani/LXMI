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
- Checkpoint previo: `a481d98`, cierre de 0.8.3; se inició 0.8.4 desde worktree limpio. Los cambios 0.8.4 permanecen sin commit/push.

## Antes de integrar runtimes o contenido de mods

- [x] Registrar repositorios upstream oficiales consultados de XXMI/WWMI y licencias publicadas; revisar individualmente todo binario/recurso redistribuible antes de distribución.
- [ ] Verificar documentación y políticas aplicables del juego/runtime; no evadir anti-cheat ni controles.
- [x] Diseñar importación segura de directorio y ZIP; probar traversal, rutas absolutas, enlaces, colisiones, profundidad, tamaño y relación de compresión.
- [ ] Investigar función live e IPC de forma separada; no asumir que existe un protocolo upstream.
- [ ] Evaluar SteamOS y distribución solo después de un MVP probado.

## LXMI-0.8.4 — Context Join fixture

CERRADO para laboratorio. [Verificación](../06_Verificacion/verificacion_0_8_4.md).

- [x] Fijar fuentes Wine/Proton y separar prefix, endpoint, tool y sesión.
- [x] Construir dos PE LXMI con MinGW, hashes y staging privado.
- [x] Ejecutar mismo prefix y control de prefixes distintos, con nonce/evento y marker real.
- [x] ContextIdentity, requirements independientes y GameContextJoinPlan sin executor.
- [x] Ejercitar ambos casos desde Tauri nativo por accesibilidad; no browser-only.
- [x] Mantener Unknown de server identity y de requisitos del loader; NoDecision/gate cerrado.
- [x] Comparar metadata protegida pre/post sin diferencias; juego y managed ZZMI no usados.
- [x] Rust/frontend quality gates y versión producto 0.8.4.

### Evidencia aún necesaria; siguiente trabajo requiere autorización

- [ ] Demostrar endpoint wineserver en fixture sin confundir candidato por entorno con instancia usada.
- [ ] Precisar propiedades necesarias para el loader (evento visible no prueba permisos de proceso ni compatibility).
- [ ] Antes de proponer usar 4162040, diseñar riesgo/preflight específico: mutaciones esperables Proton, frontera backup/rollback, helper exacto y consentimiento. No ejecutar helper en ese prefix durante 0.8.4.
- [ ] Procedimiento explícito para actualizar bins propios del laboratorio cuyo hash cambie; nunca sobrescribir un artefacto diferente silenciosamente.
- [ ] Ensayar otras tools/versiones únicamente si se autoriza: no generalizar el resultado a todos los Proton/Wine.

0.9 permanece sin autorización. Hook bloqueado; Direct Inject fixture no se promueve a compatibilidad ZZMI/ZZZ.

## LXMI 0.8.5 — estado histórico previo a 0.8.5.1: PARCIAL / BLOCKED

- [x] Mismo prefix: Direct Inject real y marker/nonce de fixture propios.
- [x] Different-prefix: control con los mismos binarios/tool; filesystem visible y pipe identity no visible, resultado explícito.
- [x] Rechazar PID homónimo de otro contexto; nonce y PID del pipe antes de acceso query-only/Inject.
- [x] UI Context Lab y modelos separados; requisitos del juego no promovidos; Hook guard intacto.
- [x] Tests offline y validaciones registrados en verificacion_0_8_5.md.
- [ ] Demostrar una vinculación segura al target propio cross-prefix antes de invocar la API. No pasar PID Windows de A a B a ciegas.
- [ ] Resolver la llamada Direct Inject cross-prefix real. Un pipe inaccesible no es resultado upstream.
- [ ] Cerrar 0.8.5 y actualizar versión solo después de resolver el bloqueo. Actual 0.8.4.
- [ ] Requisitos same-prefix/wineserver del futuro loader siguen NotEstablished. ZZZ compatibility UNKNOWN; LoaderStrategy NoDecision; gate cerrado.

No iniciar 0.9 ni ejecutar helpers en compatdata/4162040. Si se necesita otra estrategia de contexto, primero documentar su identidad y alcance, sin debilitar la regla fixture-only.

## Cierre LXMI 0.8.5 mediante 0.8.5.1

- [x] Protocolo self identity con imagen de sesión y creación; child propio correlacionado sin enumeración.
- [x] Query limitada de un solo PID y gate estricto backend/runner; fixtures collision, image/time/nonce mismatch, denied/exited/not-visible.
- [x] HOST TEST same-prefix Verified + Inject PASS y cross-prefix PidNotVisible Win32 87, BLOCKED_NOT_ATTEMPTED concluyente. La API cross-prefix no se intenta, no queda requisito pendiente para cerrar el lab.
- [x] Tauri nativo muestra ambas identidades y motivo exacto; calidad y versión producto 0.8.5.
- [ ] Requisitos universales de prefix/wineserver, loader compatibility y join de contexto real siguen UNKNOWN/NotEstablished; NoDecision y gate cerrado.

No se inicia 0.8.6 ni 0.9; ningún helper ejecutado en 4162040. [Verificación final](../06_Verificacion/verificacion_0_8_5.md).

## LXMI 0.8.6 — CERRADO para reentrada fixture

- [x] Target ready antes de segunda invocación independiente de Proton.
- [x] Same-prefix: evento y binding PID/image/time/nonce Verified; different-prefix no verifica, filesystem separado.
- [x] Prefix dev/inode, endpoint DERIVED/socket observado, registry/mapping metadata inventory y server PID UNKNOWN.
- [x] GameContextJoinPlan y riesgo de entry declarativos; UI nativa/quality/version 0.8.6.
- [ ] Reentrada al contexto del juego y mutaciones/rollback exactos requieren preflight y consentimiento propios; no ejecutar en 4162040.
- [ ] Endpoint wineserver exacto de peer y compatibilidad loader/game siguen UNKNOWN; strategy NoDecision/gate cerrado.

No se inicia 0.9 ni ningún siguiente incremento automáticamente. Regresión late Direct Inject opcional no ejecutada en este probe sin loader.

## LXMI 0.8.7 — revisión inicial histórica; NEEDS EVIDENCE

- [x] Preflight read-only, identidad prefix, build drift, actividad procfs same-UID, baseline selectivo con hash y panel Tauri nativo.
- [x] Registrar cobertura limitada de paths protegidos; MetadataOnly no se presenta como backup.
- [ ] Establecer y probar backup/restauración suficiente para el plan real; blocker `rollback_insufficient_metadata_only`.
- [ ] Alcanzar EligibleForReview; mientras tanto versión 0.8.6 y secuencia detenida.
- [ ] 0.8.8 requiere gate previo y consentimiento explícito posterior; no iniciado.
- [ ] 0.8.9 requiere éxito/ mutaciones aceptables de 0.8.8; no iniciado.
- [ ] 0.9 no autorizado.

Ver `01_Descubrimiento/game_prefix_entry_preflight_0_8_7.md` y `06_Verificacion/verificacion_0_8_7.md`.

## Cierre LXMI 0.8.7 mediante 0.8.7.1 — ELIGIBLE FOR REVIEW

- [x] Backup completo soportado pfx en storage privado LXMI, no dereference de symlinks, metadata/hash manifest y staging exclusivo.
- [x] HOST TEST: fuente inactiva/identidad estable, 2445 entradas verificadas y consistencia before/after dentro de política.
- [x] Restore sandbox VERIFIED OFF-TARGET; 783 archivos/2445 entradas sin diferencias. Real-prefix restore NOT TESTED ON REAL PREFIX, sin executor/botón.
- [x] RecoverableBackupVerifiedOffTarget; reevaluación nativa EligibleForReview sin eliminar otros blockers ni autorizar entry.
- [x] Quality gates y producto0.8.7. Documentación y ADR-030 con cobertura/limitaciones.
- [ ] 0.8.8: NOT STARTED; necesita instrucción posterior, revalidación, consentimiento explícito y riesgo de mutaciones entendido.
- [ ] Restauración real, atime/ctime/inodes/hardlink topology, dependencias externas y efectos fuera de pfx no probados/cubiertos.
- [ ] Gestión futura de retención últimos2+activo y cleanup manual; no deletion automática ni sobrescritura.
- [ ] 0.8.9/0.9 no iniciados; loader strategy NoDecision, game gate cerrado y compatibilidad ZZMI/ZZZ UNKNOWN.

[Verificación](../06_Verificacion/verificacion_0_8_7.md). No ejecutar nada dentro de4162040 para ampliar esta evidencia automáticamente.
