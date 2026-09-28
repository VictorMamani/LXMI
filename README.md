# Proyecto 19 — LXMI

**LXMI** significa *Linux Model Importer & Live Mod Manager* (nombre de trabajo, según la propuesta inicial). Es un único proyecto para explorar una aplicación de escritorio Linux que ayude a preparar runtimes XXMI compatibles con Wine/Proton y administrar mods locales. El nombre, el alcance y la compatibilidad todavía son propuestas.

## Estado

- **Etapa:** construcción; LXMI-0.1, 0.2 y 0.3 implementados. La investigación de necesidad y compatibilidad sigue abierta.
- **Tipo de proyecto:** herramienta técnica de escritorio y posible proyecto de portafolio; no hay validación de demanda, modelo de ingresos ni rentabilidad confirmada.
- **Base de trabajo:** idea aportada por el usuario. Las estructuras de Proton consultadas se contrastaron con fuentes de Valve; esto no valida compatibilidad del juego, mods, ni políticas aplicables.
- **Código:** aplicación Tauri 2 con React/TypeScript y workspace Rust. Los escaneos de Steam, juegos y herramientas de compatibilidad son de solo lectura.

## Concepto

LXMI se mantiene como un proyecto con dos subsistemas relacionados:

1. **Runtime Manager:** detectar instalaciones y ayudar a configurar XXMI/WWMI/ZZMI en Wine o Proton. La primera etapa administraría instalaciones y configuración; no reimplementaría el runtime gráfico.
2. **Mod Manager:** biblioteca local, instalación segura de archivos, activación/desactivación y perfiles. Más adelante se investigaría un administrador live y un puente IPC, sujeto a viabilidad técnica y permisos.

La posible separación en otro repositorio solo se evaluaría si en el futuro se modifica o mantiene un fork de código upstream. No forma parte del arranque.

## Incrementos de descubrimiento

LXMI-0.1 prepara una aplicación Tauri 2 con React/TypeScript y Rust, muestra información básica del sistema, localiza Steam y lista las bibliotecas de `libraryfolders.vdf`. LXMI-0.2 lee manifests `appmanifest_*.acf`, enumera las aplicaciones válidas, identifica Wuthering Waves por su AppID oficial y consulta `compatdata/<AppID>/pfx` de manera pasiva. LXMI-0.3 descubre herramientas de compatibilidad en Steam Libraries y `compatibilitytools.d`, y diferencia Proton de Steam Linux Runtime mediante metadata estructural. Los tres incrementos son de solo lectura.

El AppID `3513350` se verificó en la [página oficial de Wuthering Waves en Steam](https://store.steampowered.com/app/3513350/Wuthering_Waves/). El descubrimiento de Proton sigue la estructura documentada por [Valve Proton](https://github.com/ValveSoftware/Proton) y su [plantilla de `compatibilitytool.vdf`](https://github.com/ValveSoftware/Proton/blob/proton_11.0/compatibilitytool.vdf.template). Detectar un manifest o una carpeta `pfx` no demuestra que el juego sea compatible con Linux/Proton ni que use un prefix sano. LXMI no determina qué Proton seleccionó Steam para Wuthering Waves. No se instalarán runtimes, mods ni juegos, ni se modificarán configuraciones.

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

`cargo check --workspace` compila también el adapter Tauri; las bibliotecas Linux necesarias están presentes. Pasaron `cargo fmt`, Clippy con `-D warnings`, los tests Rust del workspace y las validaciones del frontend. La app Tauri se inició y su ventana se revisó visualmente; el flujo de escaneo mostró los resultados e incidencias locales.

El scanner ejecutado en solo lectura contra Steam local encontró Wuthering Waves (AppID `3513350`, carpeta presente, `compatdata` no encontrada), Proton Experimental y tres herramientas Steam Linux Runtime. También informó y omitió un symlink llamado `Steam.dll`; no se siguió ese enlace. La ventana mostró los runtimes y la incidencia. Esta observación verifica discovery del filesystem, no lanzamiento, selección del runtime, compatibilidad del juego o XXMI.

## Siguiente resultado

LXMI-0.4: modelar una planificación de runtime/lanzamiento a partir del juego, compatdata y herramientas disponibles, sin lanzar procesos ni modificar configuraciones hasta definir y verificar ese alcance.

## Estado local de Git

El repositorio efectivo es esta carpeta LXMI, rama `main`, remote `origin` en `git@github.com:VictorMamani/LXMI.git`. El árbol estaba limpio al iniciar este incremento. Los cambios actuales aún no se han confirmado ni publicado.

## Criterio de honestidad

Los nombres de tecnologías, juegos, runtimes, versiones y etapas que aparecen en estos documentos son una dirección inicial propuesta, no una declaración de compatibilidad comprobada. Los requisitos y roadmap podrán cambiar con los hallazgos.
