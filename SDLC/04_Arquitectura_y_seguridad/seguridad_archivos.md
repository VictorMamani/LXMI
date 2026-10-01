# Seguridad de archivos y límites de integración

## LXMI-0.1 a 0.4: detección y planificación de solo lectura

- Los incrementos solo consultan metadata y leen `libraryfolders.vdf`, `appmanifest_*.acf`, metadata de compatibility tools y rutas candidatas de `compatdata`; no modifican Steam, juegos, Proton ni prefixes.
- Rutas candidatas se limitan a directorios conocidos bajo home/XDG; su procedencia queda visible en resultados/diagnósticos.
- Resolver un alias simbólico de Steam sirve para canonizar y deduplicar la detección; no se escribe ni se elimina contenido siguiendo ese enlace.
- El parser KeyValues procesa texto local, limita cada archivo a 2 MiB y la profundidad anidada a 64 niveles, y devuelve errores con offset; no interpreta contenido como comando.
- `installdir` debe ser un nombre de directorio relativo simple. El scanner rechaza manifests con rutas absolutas, separadores, `..`, AppID inválido o discrepancia entre AppID y nombre del archivo.
- El scanner no sigue symlinks en manifests, `steamapps/common`, rutas de juego, `compatdata` ni `pfx`. Un `pfx` presente se informa como candidato, no como prefix sano o Proton activo.
- Proton discovery solo lee `toolmanifest.vdf`, `compatibilitytool.vdf`, `proton` como metadata de archivo, `version` y `VERSIONS.txt`; no ejecuta entrypoints ni sigue symlinks.
- `install_path` custom no acepta `..`, componentes de prefijo ni rutas relativas con root. Los paths absolutos admitidos por el formato se validan como directorios reales, recorriendo componentes con `symlink_metadata` y rechazando enlaces. No se escribe en esos paths.
- Un nombre de carpeta no es prueba suficiente de runtime. La clasificación usa layer metadata y archivos estructurales; los Steam Linux Runtime se mantienen separados de Proton.
- `lxmi-runtime` combina resultados ya detectados para formar un plan efímero. No abre rutas ni ejecuta procesos; runtime discovery no implica selection y selection no implica launching.
- Los tests usan directorios temporales, no recorren ni modifican bibliotecas reales. El scan de integración local es independiente y solo lectura.

## Archivos de mods — trabajo futuro

LXMI 0.5 aún no implementa una biblioteca de mods ni activación/desactivación. Las reglas de gestión de mods que siguen son criterios para incrementos futuros.

## LXMI-0.5 — paquetes XXMI y WWMI

- Las fuentes locales son entrada no confiable. LXMI importa directorios, no ZIP/7z/RAR; la extracción externa no la proporciona el importador.
- Contrato actual acotado a WWMI `WWMIv1` comprobado en el release WWMI v1.0.0, ZZMI por anclas estructurales verificadas en upstream y XXMI Libraries como paquete separado. El contrato no es un inventario exhaustivo, intérprete INI, verificación de firma ni prueba de ejecución.
- La inspección es pasiva, finita y read-only sobre candidatos explícitos. No busca recursivamente todo el disco, no ejecuta DLL/EXE/script y no escribe en game/prefix/Steam.
- SHA-256 cubre ruta/tamaño/digest de cada archivo y se revalida antes de listar/planificar. Detecta cambios accidentales posteriores; no autentica el publisher. `Manifest.json.signatures` se conserva como declaración upstream y no se verifica con la clave upstream.
- Límites por defecto: 4096 entradas, profundidad 24, 128 MiB por archivo y 512 MiB total. No se implementa ZIP bomb mitigation todavía porque no se abre ningún archive. Los límites son del importador de directorios; revalidación del managed payload aplica los mismos límites.
- Se rechazan rutas absolutas, traversal, separadores ambiguos, nombres de control, colisiones case-insensitive, symlink, hardlink y entradas no regulares. Almacén raíz y ancestros se abren sin seguir symlink; salida se ancla a directorios LXMI privados y promoción no reemplaza IDs existentes.
- Fuente y almacenamiento no pueden solaparse. El servicio Tauri comprueba separación con Steam, libraries, game y compatdata observados antes de import. Cuando Steam discovery no da límites confiables, se bloquea import. Root se resuelve solo desde `SystemInfo.xdg_data_home`; no viene del frontend.
- Staging incompleto nunca se cuenta como package válido. Error limpia su temp; tras crash puede persistir temporal desconocido que necesita inspección manual antes de limpieza. `packages/xxmi/<digest>` solo se considera verificado tras volver a calcular inventory y comparar manifest.
- Un usuario o malware que corre con los mismos permisos de la cuenta puede alterar su propio storage. 0.5 no pretende ser sandbox contra procesos de la misma cuenta.
- La promoción usa rename no-replace y syncs básicos, pero no es una operación transaccional resistente a corte eléctrico demostrada. 0.6 requiere journal durable, backup/rollback ensayado y recuperación crash.
- Packages ausentes, runtimes ausentes, no compatibles aún y versiones desconocidas son observaciones/estados; no son errores fatales. Invalid metadata, permiso denegado, checksum mismatch y unsafe path sí son diagnósticos diferenciados.

Threats incluidos: malicious path names, symlink/hardlink escape, overwrite, duplicate/case collision, corrupt bytes, huge directory, stale metadata. No resueltos aquí: filesystems hostiles remotos, agotamiento por millones de directorios sobre límite (se detiene al exceder), crash/power loss, firma/origen, races de otro proceso con la misma cuenta, malware dentro del contenido en una eventual integración runtime.

La revisión upstream identificó declaraciones GPLv3, MIT/BSD/PCRE2 y contenidos que pueden tener otros titulares/licencias. No distribuir paquetes/recursos de terceros desde LXMI hasta cerrar matriz y obligaciones. Revisar `01_Descubrimiento/ecosistema_xxmi_wwmi.md` y ADR-021.

## LXMI-0.5.1 — ZZZ y ZZMI

- El scanner identifica Zenless Zone Zero por el AppID del manifest Steam `4162040`, deriva el directorio desde `installdir` y busca únicamente los nombres de ejecutable confirmados en el registro. La inspección no ejecuta el EXE, no sigue symlinks ni implica que la instalación esté completa.
- `ZZMI` solo se asocia a `ZenlessZoneZero`; `WWMI` solo a `Wuthering Waves`. Rechazar una asociación cruzada es una regla de dominio, no una medida de compatibilidad de plataforma.
- La validación ZZMI inspecciona una lista estructural mínima contrastada con el árbol upstream. Se trata como entrada local no confiable: no interpreta INI, no ejecuta scripts/binarios y no verifica la autenticidad de la release.
- `XXMI Libraries` se representa como una dependencia distinta. Una carpeta ZZMI estructuralmente válida no demuestra que las bibliotecas requeridas estén disponibles ni que la combinación pueda ejecutarse.
- El assessment mantiene separados `game_support`, distribución detectada, soporte de Steam, compatibilidad Linux/Proton, selección de Proton y estado del prefix. Steam/Linux/Proton permanecen `Unverified` en ausencia de evidencia de ejecución autorizada.
- El plan solo describe operaciones propuestas hacia el almacenamiento administrado por LXMI. No contiene apply ni destinos dentro de ZZZ, Steam o `compatdata`.

**Límite de autenticidad:** los hashes SHA-256 del almacenamiento LXMI comprueban que el contenido coincide con el inventario creado al importar; no validan la firma upstream de ZZMI. La metadata de firma que usa XXMI Launcher no ha sido implementada en LXMI.

## Controles requeridos antes de archives y activación de mods

Estos controles no implican soporte actual de archivos comprimidos ni instalación de mods. En 0.5/0.5.1 se importan únicamente directorios locales al almacenamiento LXMI descrito arriba.

- Tratar cualquier archive futuro como entrada no confiable.
- Inspeccionar nombres de entradas antes de extraer: rechazar rutas absolutas, `..`, separadores ambiguos, enlaces simbólicos/duros no autorizados, nombres duplicados y colisiones.
- Anclar el destino a un descriptor/directorio controlado y demostrar que cada ruta permanece dentro de él.
- Limitar cantidad de entradas y tamaño descomprimido para evitar agotamiento de disco/memoria.
- Extraer primero a directorio temporal nuevo; validar antes de promoverlo a la biblioteca.
- No seguir enlaces al activar/desactivar ni borrar recursivamente una ruta que salga del área de LXMI.
- Nunca editar el original. Guardar hash y ubicación de fuente; escribir transformaciones en área gestionada.
- Antes de reemplazar configuración del runtime, crear backup y comprobar que se puede restaurar.

## Procesos y permisos

- No requerir root para flujos normales.
- No concatenar argumentos del usuario en una shell. Usar APIs de proceso con ejecutable y argumentos separados.
- Mostrar y registrar claramente qué ejecutable, prefix, argumentos y variables se usarán.
- Validar rutas permitidas y evitar ejecutar binarios incluidos en un archivo de mod.
- Redactar rutas sensibles y evitar registrar tokens, cookies o variables secretas.

## Runtime, juegos y distribución

Antes de integrar o distribuir, revisar repositorios oficiales, licencias, avisos de copyright y políticas aplicables del juego/runtime. No modificar anti-cheat, DRM ni controles, no simular compatibilidad y no automatizar acciones que puedan poner una cuenta en riesgo. Si los límites no están claros, mantener pruebas en fixtures o juegos de laboratorio autorizados.

La licencia de LXMI no debe elegirse hasta conocer dependencias y componentes que se empaquetarían. Este documento no es una conclusión legal.

## LXMI-0.5.2 — releases y ZIP

- La consulta/descarga requiere acción explícita. El provider admite solo `leotorrez/ZZMI-Package` y `SpectrumQT/XXMI-Libs-Package`; pruebas unitarias usan provider fixture offline.
- Metadata, descarga y package administrado se separan. La provenance guarda repositorio, ID/tag/commit, URL/asset, fechas de publicación/consulta/descarga, firma, digest esperado/observado y companion asset.
- Solo HTTPS hacia API GitHub y hosts de assets permitidos. Redirects se siguen manualmente con máximo 5 y se revalida cada destino. Se bloquean esquema local, IP literal, userinfo, puerto no estándar y host no allowlisted. Metadata se limita a 1 MiB; los ZIP a 32 MiB y companion JSON a 64 KiB; timeout de conexión 10 s y total 30 s.
- El stream se guarda como `.partial` en `XDG_CACHE_HOME/lxmi/downloads` (fallback `~/.cache/lxmi/downloads`), con límite real de bytes y SHA-256. Solo se promueve cuando termina, coincide el tamaño del API y, si GitHub publica digest, también coincide ese hash.
- Firma de release se verifica sobre bytes exactos con las claves upstream fijadas y ECDSA P-384/SHA-256. Firma ausente/inválida/algoritmo no soportado bloquea la importación oficial. Para XXMI Libraries, `Manifest.json` separado se coteja con el digest GitHub y sus firmas de `3dmloader.dll`, `d3d11.dll` y `d3dcompiler_47.dll` se verifican criptográficamente. El documento Manifest no tiene firma propia; no describirlo como firmado.
- ZIP se extrae con límites a staging privado: 4096 entries, profundidad 24, 128 MiB por archivo, 512 MiB descomprimidos, compresión hasta 1000:1. Rechaza traversal, absolute/Windows paths, separadores ambiguos, colisiones case-insensitive, enlaces y tipos especiales; stream valida tamaños y CRC. RAR/7z no están soportados.
- Solo staging ya validado puede promoverse a `$XDG_DATA_HOME/lxmi/packages/xxmi/<digest>` con no-replace. La operación de release no escribe al juego, Steam, compatdata o prefix ni ejecuta contenido.
- SHA-256 es integridad; la firma upstream vincula el byte sequence con la clave fijada; ninguna indica que el archivo sea seguro al ejecutar, compatible o distribuible. El store vuelve a comprobar su inventario local; no vuelve a verificar la firma del ZIP al listar si el ZIP original no se conserva en el managed package.
- Riesgo residual: límites altos pueden consumir espacio; descompresor/OS y procesos del mismo usuario quedan fuera de un sandbox; DNS/TLS se confían dentro de la allowlist; licencias por componente y ZZZ Steam/Linux/Proton siguen sin resolver. Apply permanece ausente.

## LXMI-0.5.3 — ensamblado privado y topología Proton

- El único destino de escritura nuevo es el storage privado XDG de LXMI bajo `lxmi/runtimes/zenless-zone-zero/zzmi/<runtime-id>`. Antes de ensamblar, el servicio comprueba que el root no se solape con Steam root/library, instalación del juego, compatdata o `pfx`; el root no lo proporciona la UI.
- El plan se resuelve antes de escribir. El ensamblado revalida los packages autenticados y sus SHA-256, escribe en un subdirectorio privado de staging, genera la configuración derivada allí y promueve sin reemplazar un runtime existente. Los packages de origen no se editan. `runtime-manifest.json` queda al lado de `ZZMI/`, fuera del payload upstream.
- `d3dx.ini` fuente se lee como datos UTF-8; la copia derivada modifica el target de proceso a partir de la observación Steam/executable y conserva `loader = XXMI Launcher.exe`. No se evalúa el INI ni se ejecutan sus referencias. Las DLL copiadas provienen del package XXMI Libraries autenticado; `3dmloader.dll` queda en el package gestionado y nunca se ejecuta.
- El assembler no escribe dentro de la biblioteca de Steam, ZZZ, `compatdata` ni `pfx`. Las pruebas host-only comparan rutas candidatas inmediatas del juego y el reporte `dosdevices` antes/después; esta comprobación puntual no es un backup ni una auditoría del árbol entero.
- `dosdevices` se lista de forma no recursiva; se aceptan solo nombres de letra de unidad `x:`. `symlink_metadata` y `readlink` registran targets, sin seguirlos recursivamente. El mapper inspecciona componentes Linux y rechaza symlinks/inaccesibilidad; un resultado `Mapped` solo expresa correspondencia lexical con un drive observado, no carga real dentro de Proton.
- La selección de Proton, estrategia loader/helper, identidad de proceso requerida y requisito de mismo prefix permanecen `Unknown`. No hay launch executor, inyección, cambio de launch options, anti-cheat bypass ni escritura al prefix.
- El runtime ensamblado incluye archivos upstream ejecutables/DLL como bytes inertes. No se ejecutan durante import, ensamblado, discovery, hash o validación. Una futura ruta de helper requerirá revisión separada de licencia, proceso, IPC y política de seguridad.

Riesgos residuales: procesos bajo la misma cuenta pueden leer/modificar el storage propio; la promoción no es una transacción resistente a pérdida abrupta de energía demostrada; un mapping Windows observado no demuestra compatibilidad de runtime; el nombre de proceso permitido por upstream aún referencia a `XXMI Launcher.exe`. No distribuir ni ejecutar el ensamblado con ZZZ hasta resolver estas incertidumbres en un entorno autorizado.

## LXMI-0.6 — frontera del Windows bridge

- El helper Windows es un probe de almacenamiento y protocolo; no contiene enumeración de procesos, carga de DLL, hooks ni APIs de memoria de otros procesos.
- El backend acepta solo un Proton candidate válido que vuelve a encontrar en el scan actual. Nunca ejecuta shell ni acepta una orden arbitraria desde React.
- Las únicas rutas enviadas al helper son relativas al root de datos LXMI; el helper deriva el root desde su ubicación, rechaza traversal/symlinks y solo lee runtime-manifest.json con límite de tamaño.
- El helper se valida por hash local contra su sidecar de build y se reporta además el hash que calcula sobre sí mismo. Esto es integridad local, no autenticidad. El nonce correlaciona mensajes, no autentica.
- Entrada JSON: máximo 64 KiB. Salida del proceso: máximo 1 MiB por stdout/stderr con drenado acotado. Deadline: 20 s; el executor termina el grupo de procesos creado por LXMI.
- El child inicia con entorno limpio y una allowlist mínima. No propaga WINEPREFIX, WINEDLLOVERRIDES ni variables PROTON_* heredadas.
- Proton solo recibe el test CompatData privado de LXMI. El executor rechaza el modo de compatdata de juego. El UI requiere aceptar que Proton puede inicializar este prefix y ejecutar mantenimiento dentro de su propia distribución.
- El test host 0.6 solo ejecutó el helper inocuo con un prefix LXMI aislado y leyó un marker bajo storage administrado. No inició el juego ni prueba carga de DLL, injector o compatibilidad ZZMI. Proton escribió en el prefix de prueba y actualizó el timestamp de `dist.lock` de su propia distribución.

Amenazas residuales: una release/versión futura de Proton puede cambiar CLI o efectos; Proton es un proceso confiable externo y no está sandboxed por esta crate; procesos del mismo usuario pueden alterar storage entre validación y lectura; el sidecar del helper y binario no ofrecen raíz criptográfica remota. Mantener la función avanzada desactivada por falta del helper o si falla cualquier hash/política.

## LXMI-0.7 — límite del Loader Lab

- El backend solo produce un `LoaderExperimentPlan` con target `lxmi-loader-test-target.exe`, DLL `lxmi-loader-test.dll`, marker bajo `tests/loader-v1/results/` y timeout de 30 segundos; estas rutas no se reciben del frontend.
- El validador rechaza destinos fuera del root administrado LXMI, targets distintos del fixture y modos/candidatos no permitidos. Solo un `ValidatedLoaderExperimentPlan` privado genera el argv fijo `proton runinprefix <runner> <mode-cerrado> <nonce>`.
- El runner puede crear únicamente el ejecutable fijo de test y llama al export upstream con el PID de su propio `CreateProcessW`. No enumera procesos, no acepta PID externo ni tiene fallback. El test DLL comprueba el proceso anfitrión y el nonce.
- El `3dmloader.dll` upstream se copia al laboratorio solo tras verificar provenance/firma del package XXMI Libraries y SHA-256. Es uso local de prueba y no se incorpora al repositorio ni se redistribuye.
- El modo Hook global, el ejecutable upstream `3dmloader.exe`, el launcher, juegos y componentes anti-cheat no se cargan ni inspeccionan. El experimento no interactúa con ZZZ, Steam o HoYoPlay.
- Proton puede crear/modificar únicamente el prefix de test `$XDG_DATA_HOME/lxmi/test-prefixes/loader-v1/compatdata` y mantener su propia distribución. La UI requiere aceptación explícita. `compatdata/4162040` no se usa.
- Salida y logs tienen límites, timeout, marker, identidad, nonce, hash del loader y rutas Windows observadas se validan antes del éxito.

### LXMI 0.8 — Hook global y ZZZ readiness

- El Hook upstream no se ejecuta desde Loader Lab. El backend rechaza el modo `hook` con `HookScopeNotIsolated` antes de acceder a rutas/procesos; un selector React no puede eludir esta validación.
- La razón es el orden de upstream: `HookLibrary` prepara el hook antes de iniciar el target y no recibe el nombre del target como argumento; ese nombre se usa después en `WaitForInjection` y en el process tracker. El target de `d3dx.ini` no constituye evidencia de aislamiento de la instalación del hook.
- No se implementa un hook/injector propio, enumeración de procesos, fallback a otro target, ni interacción con ZZZ, Steam, prefix de juego o anti-cheat.
- El readiness enumera capacidades y evidencias, separa el Proton del laboratorio del Proton del juego y usa un gate declarativo sin executor. Hook bloqueado, Steam/Proton no verificado, mismo prefix desconocido o compatibilidad específica no habilitan el lanzamiento.
- El binario local `3dmloader.dll` se inspeccionó por sus imports PE para `SetWindowsHookExW` y `UnhookWindowsHookEx`; no se ejecutó, parcheó ni desensambló como parte de 0.8.

El resultado demuestra solo la carga del DLL de test dentro del proceso de laboratorio bajo la combinación probada. No verifica el Hook default de ZZMI, ZZZ Steam, Linux/Proton para el juego ni compatibilidad con anti-cheat. Ver `../06_Verificacion/verificacion_0_7.md`.

### LXMI 0.8.2 — lectura de Steam config

- `config/config.vdf` se resuelve desde una Steam root devuelta por discovery y asociada a la biblioteca que contiene el manifest del AppID. No hay path ingresado por frontend.
- La lectura está limitada a 2 MiB y acepta únicamente un `config/` directory y `config.vdf` regular; un symlink en cualquiera de esos componentes se clasifica como `UnsafePath` sin seguirlo.
- Se reutiliza el parser Valve KeyValues existente, con detección de secciones/campos duplicados y error tipado para estructura inesperada. Un archivo ausente, ilegible o inválido produce evidencia `Unknown`, no “sin override”.
- La sección por AppID y el mapping global `0` se conservan como datos distintos. Un mapping global nunca rellena `selected_game_runtime`.
- El scanner no escribe Steam; no consulta procesos y no ejecuta Proton/Wine/wineserver/ZZZ. La presencia de `compatdata/4162040/pfx` sigue siendo evidencia de filesystem, no evidencia de runtime activo.

### LXMI 0.8.3 — observación de procfs

- El inicio requiere consentimiento explícito que confirma que el usuario ejecutará ZZZ manualmente desde Steam. El backend vuelve a construir AppID, nombre/path ejecutable, compatdata esperado y Proton candidates desde discovery; la UI no puede enviar una ruta, PID, UID ni proceso arbitrarios.
- Solo se observa procfs Linux y procesos cuya metadata de directorio y UID efectivo corresponden al usuario actual. La sesión tiene 90 segundos máximos, muestreo de 1 segundo y lista limitada de ancestros. El único control disponible detiene el thread observador; no señala procesos del sistema.
- Se leen `status`, `stat`, `comm` para indexar procesos del UID actual. Para candidatos relacionados se leen `exe`, `cmdline` y un subconjunto exacto de environment. Las claves permitidas son `STEAM_COMPAT_APP_ID`, `SteamAppId`, `SteamGameId`, `STEAM_COMPAT_DATA_PATH` y `WINEPREFIX`; no se devuelve environment ni cmdline completos.
- No se abre `/proc/<pid>/mem`; no se usa `ptrace`, `process_vm_readv`, debugger/tracing attach, ni APIs de señal. No se abre ni manipula el proceso observado. El timestamp de inicio del proceso se coteja antes/después de la lectura para descartar reutilización obvia de PID.
- Los datos son efímeros en memoria, no se escriben a logs ni almacenamiento. El snapshot frontend redacta el home de los paths de compatdata/prefix/tool; nombres de proceso, roles y evidencia estructural no incluyen PID seleccionable o path arbitrario.
- Un proceso con `WINEPREFIX` coincidente no se declara wineserver efectivo del juego: relación de endpoint, same-prefix/same-wineserver del futuro loader, compatibilidad ZZMI/ZZZ y gate permanecen desconocidos.
- Los errores de permiso en `environ` se conservan como `PermissionDenied`, separados de otros `Unavailable`; el resto de evidencia independiente puede seguir mostrándose. Los contadores distinguen candidatos directos de contextos aceptados que incluyen ascendencia. Múltiples candidatos producen `AmbiguousProcess` y no se escoge uno.

La revisión del código y los fixtures se complementó con observación manual en la ventana Tauri nativa. El proceso observado y los límites/unknowns están registrados en `../01_Descubrimiento/observacion_runtime_zzz_steam_0_8_3.md` y `../06_Verificacion/verificacion_0_8_3.md`.

### LXMI 0.8.1 — evaluación de estrategia y aislamiento

- La auditoría upstream fijada confirma que `use_hook=true` es default de ZZMI, y que Custom Launch con modo `Inject` llega a la rama genérica Direct Inject. El default de ZZMI no se altera en LXMI y ninguna opción UI habilita ejecución del juego.
- `DirectInjectCandidate` es únicamente una hipótesis de siguiente prueba: combina ruta upstream documentada, capacidad y DLL remota comprobadas en fixture. No equivale a soporte de ZZMI, ZZZ, Steam Linux o Proton.
- Wine almacena hooks globales en la tabla del desktop dentro de una sesión/server según el source consultado. El estado se representa como `WineDesktopWithinServer`; no se eleva a `ProcessBound`, y no prueba que Proton o `3dmloader.dll` tengan idéntica implementación.
- Distintos `WINEPREFIX` permiten sesiones Wine independientes en la documentación general, pero same-prefix/same-wineserver para el loader y ZZZ sigue `Unknown`. No se ejecutó Hook para medir ese comportamiento.
- `HookRequirementState::NotEstablished` no puede usarse como evidencia de que Hook no sea necesario. `LoaderStrategyDecision.outcome=NoDecision`; readiness `NeedsEvidence`; gate no elegible.
- El servicio conserva `hook_scope_not_isolated` como la primera condición del command Hook. No cambia el modelo de amenazas ni relaja la política de ejecución.

La comparación upstream y fuentes están en `../01_Descubrimiento/estrategia_loader_zzmi_linux.md`; la verificación está en `../06_Verificacion/verificacion_0_8.md#lxmi-081`.

## Context Lab 0.8.4

Solo se copian PE authored LXMI y hashes desde outputs de build controlados. No hay binary upstream ni loader en esta prueba. Se rechazan artefactos modificados/symlinks y roots Steam/steamapps/compatdata/pfx; paths de ejecución y markers los deriva el backend en una sesión privada (0700) con nonce. No se aceptan paths/PIDs/DLLs del frontend. El staging existente diferente requiere revisión en vez de overwrite silencioso. No se altera managed runtime/packages.

Solo se inspeccionan metadatos de wineserver del UID actual y WINEPREFIX allowlisted; un match se conserva como candidato, nunca como server endpoint probado. Peers usan handles de evento, no procesos, y no ejecutan red/registry/loader/memory APIs. Env de ejecución se construye desde allowlist, HOME/XDG privados y compatdata nuevos. Proton/Wine puede escribir prefixes y sockets en /tmp/.wine-UID: esto es parte explícita del experimento. Context Lab no señaliza wineserver ni grupos; su timeout solo termina el launcher Linux hijo. Captura no bloqueante limita stdout/stderr a 1MiB cada uno, aun con pipes heredados.
