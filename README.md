# Proyecto 19 — LXMI

**LXMI** significa *Linux Model Importer & Live Mod Manager* (nombre de trabajo, según la propuesta inicial). Es un único proyecto para explorar una aplicación de escritorio Linux que ayude a preparar runtimes XXMI compatibles con Wine/Proton y administrar mods locales. El nombre, el alcance y la compatibilidad todavía son propuestas.

## Estado

- **Etapa:** construcción; LXMI-0.1 a 0.5.1 implementados. La investigación de necesidad y compatibilidad de ejecución sigue abierta.
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

LXMI-0.5 integra importación de carpetas XXMI/WWMI a staging privado bajo XDG, inventario SHA-256 y plan declarativo revisable. LXMI-0.5.1 agrega Zenless Zone Zero y ZZMI, con XXMI Libraries como dependencia separada. No extrae ZIP, no verifica firmas upstream ni escribe en el juego o prefix. La investigación específica de ZZMI está en [ecosistema XXMI/ZZMI](SDLC/01_Descubrimiento/ecosistema_xxmi_zzmi.md); la investigación histórica WWMI permanece en [ecosistema XXMI/WWMI](SDLC/01_Descubrimiento/ecosistema_xxmi_wwmi.md).

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

LXMI-0.5.1 detectó en modo de solo lectura una instalación Steam de Zenless Zone Zero: manifest/AppID `4162040`, directorio presente, `ZenlessZoneZero.exe` encontrado y `compatdata/4162040/pfx` presente como candidato. El scan encontró un candidato Proton, pero la selección por juego permaneció `unknown` y la readiness `incomplete`. Esto no demuestra que ZZMI funcione con ZZZ, Steam o Linux/Proton. La prueba host-only ejecutó el mismo snapshot de scan que consume el command Tauri; no se automatizó el click en la ventana. Detalles en [verificación 0.5.1](SDLC/06_Verificacion/verificacion_0_5_1.md).

LXMI-0.4 pasó `cargo fmt`, `cargo check`, Clippy con `-D warnings`, 69 tests Rust y typecheck/lint/format/build del frontend. `npm run tauri:dev` inició la ventana Tauri y ejecutó un escaneo local en modo de solo lectura.

En el scan real de LXMI-0.4 (2026-09-27) se encontró Wuthering Waves (AppID `3513350`) con directorio presente, compatdata/pfx ausentes, dos candidatos Proton (Experimental y Hotfix), tres Steam Linux Runtime, selección desconocida y readiness `NeedsInitialization`. Ese resultado es una observación histórica del host. El escaneo quedó completo; un symlink irrelevante `Steam.dll` fue omitido sin seguirlo y como observación informativa. No se encontró una herramienta custom.

La política `Expected` de Wuthering Waves expresa una expectativa de planificación, no una afirmación de compatibilidad. La selección sigue desconocida aunque exista un candidato único. Compatdata/pfx ausentes son estados válidos y no se crean. La completitud del juego no se infiere del directorio presente. No se ejecuta Steam, Proton, Wine ni el juego.

LXMI-0.5 pasó los quality gates del workspace con 114 tests Rust (45 nuevos) y las validaciones frontend. El validador se probó con fixtures sintéticos y directorios extraídos de releases oficiales WWMI v1.0.0 y XXMI Libraries v1.1.7 en `/tmp`. El import/plan de ejemplo no ejecutó binarios ni modificó el juego. La ventana Tauri v0.5 inició; la UI visual se inspeccionó, pero el flujo IPC de importación no se recorrió clic a clic en la ventana nativa. En un nuevo scan read-only (2026-09-28), el host no mostró Wuthering Waves en los manifests de la biblioteca detectada; las carpetas Proton vistas carecían de metadata/entrypoint suficientes para validar una herramienta, y el plan quedó `Blocked`. El estado puede cambiar con Steam. Ver [verificación 0.5](SDLC/06_Verificacion/verificacion_0_5.md).

## Siguiente resultado

LXMI-0.6: aplicar una instalación segura únicamente tras resolver requisitos upstream y de compatibilidad, con destino exacto aprobado, backups, journal, rollback y verificación. No se implementó en 0.5.

## Estado local de Git

El repositorio efectivo es esta carpeta LXMI, rama `main`, remote `origin` en `git@github.com:VictorMamani/LXMI.git`. El árbol estaba limpio al comenzar LXMI-0.4; sus cambios siguen en desarrollo hasta que se confirmen.

## Criterio de honestidad

Los nombres de tecnologías, juegos, runtimes, versiones y etapas que aparecen en estos documentos son una dirección inicial propuesta, no una declaración de compatibilidad comprobada. Los requisitos y roadmap podrán cambiar con los hallazgos.
