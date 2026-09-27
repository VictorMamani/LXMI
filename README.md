# Proyecto 19 — LXMI

**LXMI** significa *Linux Model Importer & Live Mod Manager* (nombre de trabajo, según la propuesta inicial). Es un único proyecto para explorar una aplicación de escritorio Linux que ayude a preparar runtimes XXMI compatibles con Wine/Proton y administrar mods locales. El nombre, el alcance y la compatibilidad todavía son propuestas.

## Estado

- **Etapa:** construcción del primer incremento técnico.
- **Tipo de proyecto:** herramienta técnica de escritorio y posible proyecto de portafolio; no hay validación de demanda, modelo de ingresos ni rentabilidad confirmada.
- **Base de trabajo:** propuesta compartida por el usuario. Aún no se verificaron fuentes oficiales, licencias, compatibilidad de juegos/runtimes ni políticas aplicables.
- **Código:** el primer incremento se limita a una app Tauri, información básica de Linux y detección de Steam/bibliotecas en modo de solo lectura.

## Concepto

LXMI se mantiene como un proyecto con dos subsistemas relacionados:

1. **Runtime Manager:** detectar instalaciones y ayudar a configurar XXMI/WWMI/ZZMI en Wine o Proton. La primera etapa administraría instalaciones y configuración; no reimplementaría el runtime gráfico.
2. **Mod Manager:** biblioteca local, instalación segura de archivos, activación/desactivación y perfiles. Más adelante se investigaría un administrador live y un puente IPC, sujeto a viabilidad técnica y permisos.

La posible separación en otro repositorio solo se evaluaría si en el futuro se modifica o mantiene un fork de código upstream. No forma parte del arranque.

## Incrementos de descubrimiento

LXMI-0.1 prepara una aplicación Tauri 2 con React/TypeScript y Rust, muestra información básica del sistema, localiza Steam y lista las bibliotecas de `libraryfolders.vdf`. LXMI-0.2 lee manifests `appmanifest_*.acf`, enumera las aplicaciones válidas, identifica Wuthering Waves por su AppID oficial y consulta `compatdata/<AppID>/pfx` de manera pasiva. Ambos incrementos son de solo lectura.

El AppID `3513350` se verificó en la [página oficial de Wuthering Waves en Steam](https://store.steampowered.com/app/3513350/Wuthering_Waves/). Detectar un manifest o una carpeta `pfx` no demuestra que el juego sea compatible con Linux/Proton ni que use un prefix sano. No se instalarán runtimes, mods ni juegos, ni se modificarán configuraciones.

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

## Estado del primer incremento

LXMI-0.1 y la lógica Rust de LXMI-0.2 están implementados con fixtures sintéticos. Pasaron los tests de `lxmi-core`/`lxmi-steam` y las validaciones web. El command Tauri ya expone el descubrimiento ampliado, pero la ventana nativa aún no se compiló ni abrió porque faltan dependencias Linux de WebKitGTK/JavaScriptCoreGTK y libsoup.

No se inspeccionó Steam real ni se detectaron juegos. La compatibilidad con Steam, Wuthering Waves, Proton y XXMI sigue sin validarse en una instalación real.

## Siguiente resultado

Preparar las dependencias nativas de Tauri y validar la interfaz integrada en Ubuntu. Después, LXMI-0.3 debe descubrir instalaciones/versiones de Proton sin ejecutar ni modificar el juego o sus prefixes.

## Estado local de Git

La carpeta aún no es un repositorio independiente: el Git efectivo es `/home/university`, rama `master`, sin commits ni remote, y el directorio LXMI aparece sin seguimiento. No se inicializó otro repositorio ni se cambió el remote; falta decidir y preparar explícitamente el límite Git de BYTE-CX/LXMI.

## Criterio de honestidad

Los nombres de tecnologías, juegos, runtimes, versiones y etapas que aparecen en estos documentos son una dirección inicial propuesta, no una declaración de compatibilidad comprobada. Los requisitos y roadmap podrán cambiar con los hallazgos.
