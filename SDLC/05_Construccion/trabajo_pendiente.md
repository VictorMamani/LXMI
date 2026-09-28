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

## Siguiente incremento: LXMI-0.6 — Proton Launch Topology Experiment (sin inyección)

- [ ] Resolver con evidencia el proceso Windows responsable de iniciar/cargar el importer; comparar launcher portable, componentes upstream y helper propio sin ejecutar el juego.
- [ ] Comprobar en un entorno permitido qué proceso/prefix debe compartir el helper y ZZZ. Mantenerlo `Unknown` hasta obtener evidencia.
- [ ] Probar, sin modificar el prefix, la visibilidad de `App.Root` por el mapping observado y las rutas Windows derivadas; separar resolución de ruta de capacidad de carga.
- [ ] Identificar licencias y requisitos redistribuibles de cada helper/DLL; no empaquetar componentes cuyo permiso de uso/distribución no esté aclarado.
- [ ] Definir un protocolo/control boundary compatible con una futura comunicación LXMI Linux ↔ runtime Windows solo después de elegir una estrategia viable.
- [ ] Establecer si existe una prueba permitida para ZZZ Steam + Linux/Proton + ZZMI sin bypass, evasión o modificación de anti-cheat; detener la ruta si el entorno no es apropiado.
- [ ] Mantener sin implementar ejecución, inyección, modificación del juego/prefix/Steam, gestión real de mods y `apply` hasta cerrar esos puntos en un incremento posterior.

## Estado de Git

- El checkpoint anterior es `9646631`; LXMI-0.5.3 está implementado y validado técnicamente en `main`. El siguiente checkpoint debe conservarse en `origin` sin publicar cambios hasta autorización.
- Root confirmado: `04_Proyectos/19_LXMI`; remote `origin` correcto. No reorganizar el repositorio.

## Antes de integrar runtimes o contenido de mods

- [x] Registrar repositorios upstream oficiales consultados de XXMI/WWMI y licencias publicadas; revisar individualmente todo binario/recurso redistribuible antes de distribución.
- [ ] Verificar documentación y políticas aplicables del juego/runtime; no evadir anti-cheat ni controles.
- [x] Diseñar importación segura de directorio y ZIP; probar traversal, rutas absolutas, enlaces, colisiones, profundidad, tamaño y relación de compresión.
- [ ] Investigar función live e IPC de forma separada; no asumir que existe un protocolo upstream.
- [ ] Evaluar SteamOS y distribución solo después de un MVP probado.
