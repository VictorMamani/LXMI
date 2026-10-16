# Proyecto 19 — LXMI

**LXMI** significa *Linux Model Importer & Live Mod Manager* (nombre de trabajo, según la propuesta inicial). Es un único proyecto para explorar una aplicación de escritorio Linux que ayude a preparar runtimes XXMI compatibles con Wine/Proton y administrar mods locales. El nombre, el alcance y la compatibilidad todavía son propuestas.

## Estado

- **Primera versión funcional de ZZZ (2026-10-15):** el usuario confirmó que carga y funciona con **ZZZ — 12 mods (4 reparados)**. LXMI abre el juego desde Steam, prepara la carga oficial de XXMI en el contexto de HoYoPlay y el usuario pulsa Jugar. Incluye interfaz launcher, biblioteca, perfiles reversibles, ShaderFixes y herramienta de reparación de copias. Wuthering Waves y la estabilidad prolongada siguen pendientes de validación. **[Manual de uso](MANUAL_DE_USO.md)** · [Prueba confirmada](SDLC/06_Verificacion/prueba_manual_zzz_coleccion_reparada.md) · [ADR-032](SDLC/04_Arquitectura_y_seguridad/ADR/032_jugar_con_mods_hook_oficial.md).
- **Pruebas de carga ZZZ previas (2026-10-15, historial):** con Inject, DLL comprobada, pero sin cambio visual y con terminación del proceso de la carga temprana; la ejecución posterior ya no tenía ZZMI. El diagnóstico gráfico propio completó inicialización/configuración con y sin los mods. Sigue pendiente aislar el cierre en el juego real. [Resultados y límites](SDLC/06_Verificacion/verificacion_primera_carga_zzz.md). El runner es opt-in y no habilita el lanzamiento con mods en la aplicación.
- **Gestor local (2026-10-14):** biblioteca independiente para WuWa/ZZZ, importación de carpeta/ZIP, deduplicación e integridad SHA-256, perfiles persistentes y preparación de copias privadas. Verificado con IPC Tauri nativo real, reinicio y hashes. [Guía de uso y límites](SDLC/05_Construccion/gestor_mods_local.md). La aplicación reversible a runtimes privados WWMI/ZZMI está implementada. La selección sin recarga sigue pendiente. [Verificación de runtime y diseño](SDLC/06_Verificacion/verificacion_activacion_runtime.md). Este trabajo no cierra el experimento 0.8.8 ni cambia la versión 0.8.7.
- **Etapa:** construcción; versión 0.8.7. El flujo de juego con mods usa el Hook oficial y el contexto Steam de HoYoPlay observado en cada inicio. La investigación histórica 0.8.8 de entrada y restauración de prefix conserva sus registros y límites; su estado no acredita restauración real ni compatibilidad con otros entornos. [Historial de observación](SDLC/06_Verificacion/evidencias/observacion_zzz_biblioteca_externa.json).
- **Tipo de proyecto:** herramienta técnica de escritorio y posible proyecto de portafolio; no hay validación de demanda, modelo de ingresos ni rentabilidad confirmada.
- **Base de trabajo:** idea aportada por el usuario. Las estructuras de Proton consultadas se contrastaron con fuentes de Valve; esto no valida compatibilidad del juego, mods, ni políticas aplicables.
- **Código:** aplicación Tauri 2 con React/TypeScript y workspace Rust. Los escaneos de Steam/juegos/tools son de solo lectura; el import explícito solo escribe en almacenamiento XDG privado de LXMI.

## Concepto

LXMI se mantiene como un proyecto con dos subsistemas relacionados:

1. **Integración de plataforma y runtimes:** detectar juegos y describir la preparación de compatibilidad con Wine/Proton; después ayudar a configurar XXMI/WWMI/ZZMI. Proton es infraestructura de plataforma, no el dominio principal del producto.
2. **Mod Manager:** biblioteca local, instalación segura de archivos, activación/desactivación y perfiles. Más adelante se investigaría un administrador live y un puente IPC, sujeto a viabilidad técnica y permisos.

La posible separación en otro repositorio solo se evaluaría si en el futuro se modifica o mantiene un fork de código upstream. No forma parte del arranque.

## Incrementos de descubrimiento

LXMI-0.1 prepara una aplicación Tauri 2 con React/TypeScript y Rust, muestra información básica del sistema, localiza Steam y lista las bibliotecas de `libraryfolders.vdf`. LXMI-0.2 lee manifests `appmanifest_*.acf`, identifica juegos del catálogo y consulta `compatdata/<AppID>/pfx` de manera pasiva. LXMI-0.3 descubre herramientas de compatibilidad mediante metadata estructural. LXMI-0.4 construye un plan declarativo por juego, con requisitos, evidencia, candidatos disponibles, selección desconocida y readiness. Todos estos incrementos son de solo lectura.

El AppID `3513350` se verificó en la [página oficial de Wuthering Waves en Steam](https://store.steampowered.com/app/3513350/Wuthering_Waves/). El discovery de Proton sigue la estructura documentada por [Valve Proton](https://github.com/ValveSoftware/Proton) y su [plantilla de `compatibilitytool.vdf`](https://github.com/ValveSoftware/Proton/blob/proton_11.0/compatibilitytool.vdf.template). Detectar un manifest o una carpeta `pfx` no demuestra compatibilidad ni salud del prefix. Tener candidatos instalados tampoco demuestra cuál seleccionó Steam para Wuthering Waves. LXMI-0.4 no ejecuta procesos ni modifica configuraciones.

LXMI-0.5 integra paquetes XXMI/WWMI bajo XDG; 0.5.1 agrega Zenless Zone Zero y ZZMI; 0.5.2 consulta/descarga releases oficiales y las autentica. LXMI-0.5.3 resuelve `App.Root/importer_path` como un runtime administrado por LXMI fuera del juego, ensambla ZZMI + XXMI Libraries y crea un plan de topología Linux/Windows. El dry-run 0.5.2 contra la carpeta candidata del ejecutable es una comparación histórica, no un destino autorizado. No se lanza ni inyecta nada, y no se modifica juego, Steam, compatdata o prefix. La selección Proton, helper y requisito de mismo prefix siguen desconocidos; Steam/Linux/Proton no están verificados. Fuentes: [adquisición oficial de paquetes 0.5.2](SDLC/01_Descubrimiento/adquisicion_paquetes_xxmi_0_5_2.md), [topología runtime XXMI Linux 0.5.3](SDLC/01_Descubrimiento/topologia_runtime_xxmi_linux.md), y los antecedentes de [XXMI/ZZMI](SDLC/01_Descubrimiento/ecosistema_xxmi_zzmi.md) y [XXMI/WWMI](SDLC/01_Descubrimiento/ecosistema_xxmi_wwmi.md).

LXMI-0.6 agrega un helper Windows inocuo y un bridge de diagnóstico por stdin/stdout JSON, con Proton explícita y prefix aislado. MinGW enlazó el `.exe`; Proton Experimental ejecutó el helper, que devolvió el nonce, leyó el runtime marker mediante una ruta Windows y coincidió el hash. Un caso negativo devolvió JSON estructurado. La ventana Tauri nativa mostró los resultados. Esta prueba no carga XXMI ni demuestra compatibilidad de ZZZ/Steam/Linux/Proton; la selección de runtime del juego sigue desconocida. Ver [verificación del bridge 0.6](SDLC/06_Verificacion/verificacion_0_6.md).

LXMI-0.7 añade un Loader Lab avanzado aislado: target y DLL propios para Windows, más el `3dmloader.dll` oficial verificado. En Proton Experimental explícito, la ruta Direct Inject cargó la DLL de prueba desde una ubicación separada y verificó marker/nonce; también pasaron los casos de baseline, target ausente, DLL ausente y nonce incorrecto. La interacción se recorrió en una ventana Tauri nativa. La ruta default Hook de ZZMI, `3dmloader.exe`, ZZZ, Steam y la compatibilidad Linux/Proton del juego no se probaron. Ver [investigación del loader](SDLC/01_Descubrimiento/loader_xxmi.md), [ADR-027](SDLC/04_Arquitectura_y_seguridad/ADR/027_xxmi_loader_strategy_for_lxmi.md) y [verificación 0.7](SDLC/06_Verificacion/verificacion_0_7.md).

LXMI-0.8 evalúa readiness de manera declarativa y conserva un guard fail-closed para Hook porque su aislamiento a un proceso no está demostrado. LXMI-0.8.1 fija la lectura de XXMI Launcher v2.2.1: ZZMI hereda Hook como modo estándar; Custom Launch expone Hook, Inject y Bypass, y Direct Inject utiliza una ruta genérica de `3dmloader.dll`. La prueba 0.7 demuestra solo el mecanismo contra el fixture LXMI, no que ZZMI funcione con ZZZ en Steam Linux/Proton. No se estableció que ZZMI requiera exclusivamente Hook, pero eso tampoco demuestra que Direct Inject sea compatible. Por ello LXMI mantiene `LoaderStrategyDecision = NoDecision`, `GameRuntimePlan.selected = Unknown`, readiness `NeedsEvidence` y gate cerrado. Ver [investigación de estrategia](SDLC/01_Descubrimiento/estrategia_loader_zzmi_linux.md), [flujo Steam/Linux](SDLC/01_Descubrimiento/launch_zzmi_steam_linux.md), [ADR-027](SDLC/04_Arquitectura_y_seguridad/ADR/027_xxmi_loader_strategy_for_lxmi.md) y [verificación 0.8.1](SDLC/06_Verificacion/verificacion_0_8.md).

LXMI-0.8.2 lee de forma acotada y read-only `config/config.vdf`, e informa por separado mapping por AppID, default global, runtimes instalados y paths `compatdata/pfx` detectados. En la fuente local inspeccionada no se encontró mapping por AppID; eso no determina el runtime efectivo. El resultado de ese incremento está documentado en [evidencia de contexto Steam/Proton](SDLC/01_Descubrimiento/evidencia_contexto_steam_proton_0_8_2.md) y [verificación 0.8.2](SDLC/06_Verificacion/verificacion_0_8_2.md).

LXMI-0.8.3 añade observación efímera de procfs con consentimiento. En la validación manual desde Tauri, el observer detectó una ejecución Steam de ZZZ con Proton Experimental y prefix/compatdata coincidentes. Esta evidencia describe esa sesión; no actualiza la selección configurada, no demuestra compatibilidad ZZMI ni habilita lanzamiento. Ver [observación local](SDLC/01_Descubrimiento/observacion_runtime_zzz_steam_0_8_3.md) y [verificación 0.8.3](SDLC/06_Verificacion/verificacion_0_8_3.md).

## Documentos SDLC

- [Fuentes e hipótesis](SDLC/01_Descubrimiento/fuentes_e_hipotesis.md)
- [Product brief](SDLC/02_Planificacion/product_brief.md)
- [Alcance MVP](SDLC/02_Planificacion/alcance_mvp.md)
- [Requisitos funcionales](SDLC/02_Planificacion/requisitos_funcionales.md)
- [Requisitos no funcionales](SDLC/02_Planificacion/requisitos_no_funcionales.md)
- [Matriz inicial de riesgos](SDLC/02_Planificacion/matriz_de_riesgos.md)
- [Flujos propuestos](SDLC/03_Diseno/flujos_de_usuario.md)
- [Arquitectura conceptual](SDLC/04_Arquitectura_y_seguridad/arquitectura.md)
- [Decisiones técnicas pendientes](SDLC/04_Arquitectura_y_seguridad/decisiones_tecnicas.md)
- [Plan de trabajo de construcción](SDLC/05_Construccion/trabajo_pendiente.md)
- [Estrategia de verificación](SDLC/06_Verificacion/estrategia_de_calidad.md)

## Estado verificado

La validación host-only de 0.5.3 detectó ZZZ Steam/AppID `4162040`, encontró `ZenlessZoneZero.exe` y `compatdata/4162040/pfx`, e inspeccionó sus drive mappings sin escritura. Ensambló 49 archivos desde ZZMI v1.5.0 + XXMI Libraries v1.1.7 bajo `/home/university/.local/share/lxmi/runtimes/.../ZZMI`; la ruta quedó mapeada por `Z:`. La selección Proton permaneció `unknown`; no se ejecutó launcher, helper, Proton, Wine ni ZZZ. Capturas puntuales antes/después confirmaron que las rutas candidatas del juego y `dosdevices` no cambiaron. Detalles: [verificación 0.5.3](SDLC/06_Verificacion/verificacion_0_5_3.md).

LXMI-0.4 pasó `cargo fmt`, `cargo check`, Clippy con `-D warnings`, 69 tests Rust y typecheck/lint/format/build del frontend. `npm run tauri:dev` inició la ventana Tauri y ejecutó un escaneo local en modo de solo lectura.

En el scan real de LXMI-0.4 (2026-09-27) se encontró Wuthering Waves (AppID `3513350`) con directorio presente, compatdata/pfx ausentes, dos candidatos Proton (Experimental y Hotfix), tres Steam Linux Runtime, selección desconocida y readiness `NeedsInitialization`. Ese resultado es una observación histórica del host. El escaneo quedó completo; un symlink irrelevante `Steam.dll` fue omitido sin seguirlo y como observación informativa. No se encontró una herramienta custom.

La política `Expected` de Wuthering Waves expresa una expectativa de planificación, no una afirmación de compatibilidad. La selección sigue desconocida aunque exista un candidato único. Compatdata/pfx ausentes son estados válidos y no se crean. La completitud del juego no se infiere del directorio presente. No se ejecuta Steam, Proton, Wine ni el juego.

LXMI-0.5 pasó los quality gates del workspace con 114 tests Rust (45 nuevos) y las validaciones frontend. El validador se probó con fixtures sintéticos y directorios extraídos de releases oficiales WWMI v1.0.0 y XXMI Libraries v1.1.7 en `/tmp`. El import/plan de ejemplo no ejecutó binarios ni modificó el juego. La ventana Tauri v0.5 inició; la UI visual se inspeccionó, pero el flujo IPC de importación no se recorrió clic a clic en la ventana nativa. En un nuevo scan read-only (2026-09-28), el host no mostró Wuthering Waves en los manifests de la biblioteca detectada; las carpetas Proton vistas carecían de metadata/entrypoint suficientes para validar una herramienta, y el plan quedó `Blocked`. El estado puede cambiar con Steam. Ver [verificación 0.5](SDLC/06_Verificacion/verificacion_0_5.md).

## Estado actual de investigación

La carga oficial mediante Hook está integrada y comprobada en ZZZ con Steam y Proton en este equipo. La colección de 12 mods con cuatro copias reparadas cargó correctamente según los registros y la confirmación del usuario. Continúan abiertas la estabilidad prolongada, Wuthering Waves y la recarga sin reiniciar. Los experimentos anteriores de Direct Inject y de prefix permanecen documentados como antecedentes.

## Estado local de Git

El repositorio efectivo es esta carpeta LXMI, rama `main`, remote `origin` en `git@github.com:VictorMamani/LXMI.git`. El historial conserva los incrementos de investigación y la primera versión funcional del launcher y gestor local.

## Criterio de honestidad

La compatibilidad comprobada corresponde a las sesiones y combinaciones documentadas. Los resultados de ZZZ en este equipo no se extrapolan a otros juegos, versiones o entornos. Los requisitos y roadmap podrán cambiar con los hallazgos.

## LXMI 0.8.4 — Context Lab

Advanced permite probar mismo prefix y prefixes distintos con Proton explícito y peers propios. No usa compatdata 4162040 ni loaders. La relación de evento, mappings, prefix y candidato wineserver se informa por separado; no se crea un launcher de juego. [Investigación y host evidence](SDLC/01_Descubrimiento/contexto_wine_proton_0_8_4.md) · [Verificación](SDLC/06_Verificacion/verificacion_0_8_4.md).

## LXMI 0.8.5 — Direct Inject target binding

El Context Lab solo admite fixtures LXMI en prefixes privados. El target reporta su identidad; el runner consulta únicamente ese PID con PROCESS_QUERY_LIMITED_INFORMATION y compara ruta exacta y FILETIME antes de cargar el loader oficial. No PID/path/DLL arbitrarios.

Same-prefix verificó identidad y marker + nonce tras Inject 0; cross-prefix no pudo resolver el target mediante query limitada (Win32 87) y bloqueó la operación de manera concluyente. Ambos leyeron filesystem, evidencia distinta de acceso al proceso. [Verificación](SDLC/06_Verificacion/verificacion_0_8_5.md). Ningún experimento de juego está autorizado.

## LXMI 0.8.6 — Existing Context Re-entry

Context Lab inicia el target propio primero y un helper independiente después. Same-prefix verificó evento y PID/image/creation/nonce; different-prefix no verificó el contexto aunque compartía filesystem. No DLLs copiadas/cargadas ni game prefix utilizado. Endpoint candidato y dev/inode no se confunden con wineserver PID. [Verificación](SDLC/06_Verificacion/verificacion_0_8_6.md).

## LXMI 0.8.7 — Prefix Backup & Restore Qualification

Backup completo del árbol pfx sin seguir symlinks, manifiesto SHA-256 y copia recuperable verificada. Restauración independiente VerifiedOffTarget, sin ejecutar Wine/Proton ni restaurar el prefix real. El gate de preflight alcanza EligibleForReview; consentimiento/ejecución de 0.8.8 siguen fuera de este incremento. Ver [preflight](SDLC/01_Descubrimiento/preflight_prefix_zzz_0_8_7.md) y [verificación](SDLC/06_Verificacion/verificacion_0_8_7.md).
