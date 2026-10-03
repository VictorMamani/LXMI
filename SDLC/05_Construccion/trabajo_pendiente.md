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

## LXMI0.8.8 — intento consentido BLOCKED antes de ejecución

- [x] Reutilizar helper/protocolo/executor con capability limitada y bridge original aislado.
- [x] Review/consent one-shot native, preconditions y journal before/after, mutaciones fail-closed.
- [x] Tests y quality offline.
- [x] Consentimiento explícito del usuario en Tauri; intento bloqueado antes de Proton por cambio de localconfig.vdf.
- [ ] Ejecución real del helper NO COMPROBADA; no reintentar sin instrucción nueva y revisión del blocker.
- [ ] Interpretar resultado/diferencias; solo si gate PASS cerrar/versionar0.8.8 y comenzar0.8.9.
- [ ] 0.9 no autorizado; estrategia/compatibilidad siguen desconocidas.

### Blocker0.8.8 — protected_path_changed

LOCAL OBSERVED: SHA/mtime/inode de Steam userdata1416423935/config/localconfig.vdf cambiaron entre review y preflight inmediato. Identidad e inventario selectivo prefix no cambiaron. Causa UNKNOWN, ningún helper/Proton ejecutado. Se necesita investigación read-only de estabilidad/policy protegida antes de autorizar otro intento. No ignorar el cambio, no actualizar backup para hacerlo pasar, no comenzar0.8.9. Versión0.8.7.

### Investigación estabilidad localconfig — realizada, guard intacto

- [x] Probe read-only90s:1 cambio fuera de rama ZZZ; campo/autor históricos no recuperables de hashes.
- [x] Diagnóstico de error con path/campos, sin valores privados; fixture específico.
- [ ] Semántica de campos externos y autoría siguen UNKNOWN; no asumir cambio inocuo ni relajar full-file guard.
- [ ] Cualquier policy semántica distinta o nuevo intento requiere decisión explícita posterior;0.8.9 no iniciado.

### Seguimiento read-only del bloqueo0.8.8

Dos ventanas240s solapadas confirmaron una escritura semántica en un leaf de primer nivel de localconfig fuera de Software/ZZZ; rama4162040 estable. Nombre exacto/escritor/causa históricos UNKNOWN. No se deduce inocuidad ni se exceptúa localconfig. Guard intacto; helper no reintentado,0.8.9 no iniciado, producto0.8.7. Se repitieron quality Rust/frontend con resultado PASS. Véase estabilidad_localconfig_preflight_0_8_8.md.

## Estado actual0.8.8 — HOST TEST ejecutado, gate BLOCKED

- [x] Nuevo consentimiento nativo; sesión699f4768f21d30842653eb404baeb237 ejecutó helper, protocolo1/nonce/hash/exit0.
- [x] Recibos before/after verificados y UI nativa muestra UnexpectedMutation/diff.
- [x] Identidad/build/backup/protected paths sin diferencias dentro de cobertura.
- [ ] Investigar por separado contenido cambiado system.reg/user.reg y metadata dosdevices/root; no reclasificación automática ni nuevo intento.
- [ ] 0.8.8 no cierra exitosamente, versión0.8.7;0.8.9/0.9 no iniciados.
- [ ] Real-prefix restore NotTested: no rollback automático. Backup original inmutable.

Los estados NotRun/ProtectedPathChanged anteriores son históricos; el último resultado es UnexpectedMutation después de helper success. Véase verificacion_0_8_8.md.

### Investigación Twintail y blocker0.8.8

Source pinned97d626e (stable/package2.5.1); ZZMI usa hook, no prueba Direct Inject. Diff registry exacto valida backup-before/current-after; Fonts +ContainerId+empty NetworkCards explican categorías observadas. Rewrites de fonts entre run/host y usr/share apuntan a diferencia de entorno Linux/container, sin prueba de causalidad completa. Diagnóstico puro/fixed-case read-only añadido; no guard debilitado, no runtime/helper ejecutado. Próximo trabajo requerido: calificar entorno equivalente usando únicamente fixture privado antes de revisar otra entrada real.0.8.8 BLOCKED, producto0.8.7,0.8.9/0.9 no iniciados. Véase twintail_y_mutaciones_prefix_0_8_8.md.

## Diagnóstico de entorno para el blocker 0.8.8 — 2026-10-02

- [x] HOST TEST con helper existente y prefix nuevo, cambiando únicamente envoltura Linux: reproduce exactamente 135 entradas nuevas/368 rutas por tabla Fonts al salir del contenedor; el retorno las invierte y las repeticiones conservan las fuentes.
- [x] Resolver stdin descartado por pressure-vessel en el transporte fixture, sin cambiar helper Windows ni debilitar nonce/hash.
- [x] Bloquear nuevas reviews/ejecuciones reales con linux_runtime_environment_unqualified antes de staging; confirmar motivo visible en Tauri nativo sin consentimiento/Run.
- [x] Preservar recibo original y backup; reporte redactado durable, cobertura selectiva real sin cambios, quality304 PASS/11ignored.
- [ ] Calificar causa/consecuencias de ContainerId (también cambia en mismo entorno), NetworkCards vacío removido y 2 secciones no clasificadas de repetición inicial.
- [ ] Definir policy estrecha y calificar environment antes de proponer otro real-prefix entry; no aceptar todos los registry cambios ni adaptar baseline para forzar PASS.
- [ ] Demostrar equivalencia suficiente del entorno propuesto y preservar consentimiento fresco independiente. No ejecutar 0.8.9 mientras0.8.8 siga BLOCKED; versión0.8.7.

Ver ../01_Descubrimiento/entorno_linux_fixture_0_8_8.md. No loader/game execution, restore real, commit ni push en este diagnóstico.

## LXMI 0.8.8.1 — calificación del entorno Linux

- [x] Modelar Proton directo, Steam Runtime container y wrapper `run` soportado; no reconstruir/copiar environment de Steam o ZZZ.
- [x] Dos runs por estrategia desde clones privados completos de estado inicial idéntico, verificado. Helper/nonce/marker/hash/exit PASS, sin loader.
- [x] Usar Proton privado verificado: locks dentro de LXMI, bytecode desactivado; rechazar ejecutar tool instalado en este harness. Conservar evidencia preliminar y límites históricos.
- [x] Clasificar Fonts/TeX, Device ContainerId, NetworkCards omitido del hive y claves Wow6432Node RunServices/RunServicesOnce vacías. UNKNOWN permanece bloqueante, sin wildcards de subtree.
- [x] Transport request privado 0600/fresh nonce/hash/schema/age/no-symlink/bounds/one-shot descriptor/cleanup, sin más poder para el helper.
- [x] Ambas recetas container: SteamRuntimeQualified solo para fixture privado/build/payload/cobertura fijados; DirectProton: Unqualified.
- [x] Mantener backend real bloqueado, exact code `linux_runtime_environment_unqualified`, UI con evidencia histórica, límites y Run deshabilitado.
- [ ] Integrar de forma revisable un executor real con receta calificada; el actual sigue DirectProton. No se habilita por un receipt privado ni frontend override.
- [ ] Tras aprobación de esa integración, preflight inmediato y consentimiento nativo fresco para UNA entry inocua real con ZZZ cerrado; clasificar before/after sin retry ni restore automático.
- [ ] Cerrar 0.8.8 y versionar solo si ese resultado real pasa. 0.8.9/0.9 no iniciados ni autorizados por este subincremento.

## LXMI 0.8.8.2 — continuación bloqueada

- [x] Integrar receta container compartida, copia privada/pins/transporte y guard sin fallback DirectProton.
- [x] Añadir self-test privado del mismo executor y comprobaciones offline de identidad, mutaciones y consentimiento.
- [x] Conservar historia UnexpectedMutation y distinguir calificación del executor del gate del prefix.
- [ ] Resolver explícitamente `recovery_source_current_mismatch` sin sustituir backup/baseline ni restaurar automáticamente para conseguir PASS. El backup original sigue Verified; source actual no coincide.
- [ ] Solo después de un preflight realmente EligibleForReview: nueva review/consentimiento Tauri y UNA entry real calificada. No reutilizar consentimiento anterior.
- [ ] Cerrar/bump 0.8.8 solo con resultado real aceptable. No iniciar 0.8.9/0.9 en este subincremento.

- [x] HOST TEST final del executor integrado en Tauri: seed + 2 runs medidos PASS, fonts estables, solo reglas exactas conocidas, mounts RO y cleanup; copia Proton validada.
- [x] Quality Rust:335 PASS /12ignored; frontend typecheck/lint/format/build y transporte ELF compilado.

- [x] Native Tauri: Qualified visible, preflight NeedsEvidence explícito, Review/Run deshabilitados y sin consentimiento/entry real; Tauri/Vite cerrados.

## LXMI 0.8.8.3 — resultado de reconciliación

- [x] Re-verificar backup histórico y enumerar árbol source completo sin seguir symlinks; fuente consistente, identidad/build/fuente Proton iguales.
- [x] Explicar contenido modificado2 hives mediante hashes exactos del before/after DirectProton; mantener fonts ENVIRONMENT SPECIFIC históricas, reglas GUID/NetworkCards y35 cambios metadata-only.
- [x] Recheck protegido y assessment explícito:ProtectedStateChanged; diff/correlación completa archivados y UI nativa con controles disabled.
- [x] Tests synthetic de nuevas generaciones preservando historia, restore sandbox y preflight actual, sin Steam real en tests normales.
- [ ] Resolver `protected_state_needs_review`: config.vdf y localconfig1416423935 difieren del baseline0.8.7; hashes actuales ya eran los del before/after DirectProton, pero claves/autor/semántica desconocidos. No declarar Steam changes benignos por hashes.
- [ ] Solo con estado protegido aceptado y assessment UnderstoodAndAcceptable: capturar baseline actual nuevo, backup ID nuevo, verificar/restaurar off-target y reevaluar.
- [ ] Entry real posterior exige executor calificado en esa sesión, fresh checks/consent y una sola ejecución; no realizarla en0.8.8.3. Versión0.8.7, 0.8.8 abierto; no0.8.9/0.9.

## LXMI 0.8.8.4 — recuperación y estado protegido actuales revisados

- [x] Resolver protected_state_needs_review mediante semántica ACTUAL acotada, no reconstrucción inventada de claves históricas. CompatToolMapping presente, mappings4162040/global0 y launch options/overrides ausentes; language english; 5s STABLE. Writer UNKNOWN, sin bloqueo perpetuo por provenance no recuperable.
- [x] Conservar raw hashes y evidencia histórica; gates usan fingerprints vinculados a metadata y reviewbackend fresca, sin override UI. Otros paths protegidos estrictos, Unknown/launch/mappings nuevos requieren revisión.
- [x] Assessment source UnderstoodAndAcceptable; nueva baseline schema2, backup inmutable f4ea33c0c047d893319296420fd86784, integridad y source consistency Verified, restore Verified off-target; generación histórica preservada.
- [x] Native Tauri read-only/recovery: estado y dos generaciones visibles, preflight puntual EligibleForReview. Quality353PASS/12ignored y frontend PASS. No consentimiento/helper/Proton/ZZZ; aplicación propia cerrada.
- [ ] Próxima continuación: calificar executor en la sesión, preflight inmediato fresco y consentimiento nativo independiente antes de UNA entry inocua con entorno calificado; no ejecutar por esta revisión ni reutilizar proof expirado/consent anterior.
- [ ] Cerrar/bump0.8.8 solo después de esa ejecución real exitosa y before/after aceptable. Versión0.8.7; 0.8.9/0.9 no iniciados.

Las listas0.8.8.1–0.8.8.3 anteriores conservan el estado histórico de sus checkpoints; esta sección registra los blockers resueltos y el pendiente ACTUAL.

## LXMI 0.8.8 FINAL — resultado ACTUAL del intento único

- [x] Recalificar executor en sesión actual, preflight fresco, baseline/recovery/Steam/identity/build/inactividad verificados.
- [x] Consentimiento nativo NUEVO y una sola ejecución real por el usuario; helper/nonce/hash/exit0 y transport cleanup PASS. Reserva persistente consumida.
- [x] Capturar before/after y preservar UnexpectedMutation: tres Fonts sections EnvironmentSpecific; cuatro ContainerId VolatileExpected; NetworkCards/RunServices/Once sin diferencias reportadas. Protected paths, Steam semantics, identidad/build y backup verificados.
- [x] Correlación artifact-only de tabla NT: reversión exacta de hashes DirectProton a container; otras dos tablas sin match exacto, no inventar conteos ni aceptación universal.
- [x] Corregir textos de estado/version y cargar resultado guardado tras recarga con API read-only; no reactivar consentimiento/Run. Quality358PASS/12ignored + frontend completo.
- [ ] Investigar en un futuro ensayo PRIVADO una copia/control que represente estado DirectProton histórico y transición a container. No ejecutar ahora ni ampliar ignores; aceptación de esa transición e impacto funcional aún no establecidos.
- [ ] Reconciliar evidencia de source post-test antes de cualquier futura revisión: el backup Verified representa PRE-ENTRY, no el nuevo estado. No crear nuevo backup/restore para conseguir PASS en esta sesión.
- [ ] 0.8.8 permanece BLOCKED, versión0.8.7. Sin nueva entry/consentimiento/retry/restore;0.8.9/0.9 no iniciados. Esta sección sustituye el pendiente de ejecutar la primera entry calificada: ya se ejecutó y no pasó el gate.

## Actualización 2026-10-03 — transición privada desde source pre-entry

- [x] Reproducir en dos clones independientes el estado de fuentes anterior al último intento; dos runs por clone, cuatro helpers PASS.
- [x] Establecer coincidencia exacta antes/después de las tres tablas con el intento real; primer run cambia fuentes, segundo estable. Backup histórico preservado/verificado.
- [x] Evitar que steady-state Qualified autorice un estado inicial de fuentes distinto: guard backend antes de Review y del argv real; tests sintéticos, sin probar el branch real.
- [ ] Revisar un plan de calificación vinculado al estado inicial aceptado y la recuperación posterior al intento. La reproducción no autoriza entry adicional ni reclasifica el fallo como PASS.
- [ ] Aceptación funcional del cambio histórico y estado real después del intento: requieren revisión/evidencia específica; sin nueva entrada, restore ni consentimiento en esta continuación.

0.8.8 sigue BLOCKED; aplicación 0.8.7. 0.8.9/0.9 no iniciados.
## Estado actual 2026-10-03 — causa corregida y recuperabilidad actual verificada

- [x] Reconciliar source post-test/Steam, sin material UNKNOWN; UnderstoodAndAcceptable/ReviewedAndAcceptable.
- [x] Crear nueva recuperación `2b29b84bbc592fa091950bf046b3d668`, Verified/full consistency, restore Verified off-target; preservar f355 y f4 históricos.
- [x] Calificar el executor desde clone verificado de esa generación y medir primer run sin normalizar: dos helpers PASS, cero cambios font/UNKNOWN, solo ContainerId estrechos. Native Qualified/recovery seed visible.
- [x] Vincular backend a generation ID/hash/fingerprints y exigir esa capacidad para real entry; calificación genérica no basta. Refresh no duplica source que aún coincide.
- [x] Native final: baseline fresco `zzz-prefix-64dac842206897b341916bed5974b11e`, preflight EligibleForReview; Review/Run reales deshabilitados por reserva consumida intacta. Quality365PASS/12ignored y frontend PASS.
- [ ] Obtener autorización explícita para preparar otro experimento inocuo independiente, conservando el fallo original. La instrucción FINAL prohibió retry tras fallo; no eliminar la reserva ni reutilizar consentimiento.
- [ ] Solo con esa autorización: nueva revisión y consentimiento manual Tauri, preflight fresco, una ejecución consume-once/misma política estricta; detener ante fallo, sin restore automático.
- [ ] Cerrar/bump0.8.8 únicamente tras una entrada real nueva autorizada y exitosa. Permanece versión0.8.7; no0.8.9/0.9.

Esta sección es el pendiente actual; las listas anteriores conservan sus checkpoints históricos. [Evidencia y propuesta concreta](../01_Descubrimiento/transicion_fuentes_backup_privado_0_8_8.md).
