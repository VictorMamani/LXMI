# Fuentes e hipótesis

## Origen de la idea

La descripción inicial de LXMI y sus componentes fue aportada por el usuario en la conversación. Este documento organiza esas ideas; no equivale a investigación independiente.

## Hechos comprobados en este repositorio

- Se ha creado una carpeta de documentación para el Proyecto 19.
- La implementación de LXMI-0.1 a 0.4 cubre información del sistema, Steam Libraries, lectura de manifests, registro de Wuthering Waves, observación pasiva de compatdata/pfx, descubrimiento de herramientas de compatibilidad y planificación declarativa. No valida selección efectiva, lanzamiento ni compatibilidad de juegos/XXMI.
- No se recibieron enlaces upstream, código, documentación de compatibilidad ni licencias de XXMI/runtimes. La fuente oficial consultada para el AppID se registra por separado; no valida compatibilidad.

## Fuente verificada para LXMI-0.2

| Fecha | Fuente primaria | Dato comprobado | Límite |
|---|---|---|---|
| 2026-09-27 | [Ficha oficial de Wuthering Waves en Steam](https://store.steampowered.com/app/3513350/Wuthering_Waves/) | La URL de la ficha usa AppID `3513350` y el producto se identifica como Wuthering Waves. La ficha también declara uso de Anti-Cheat Expert a nivel de kernel. | Confirma identidad/AppID de Steam, no instalación local, compatibilidad con Linux/Proton, seguridad de mods ni funcionamiento de LXMI. La mención anti-cheat exige revisión antes de cualquier integración de runtime o mods; no se intentará eludir controles. |
| 2026-09-27 | [Repositorio oficial ValveSoftware/Proton](https://github.com/ValveSoftware/Proton) | Valve distribuye varias versiones de Proton con Steam y documenta herramientas custom en `compatibilitytools.d`; el layout incluye manifests y un entrypoint `proton`. | La estructura permite descubrir archivos locales; no indica qué versión selecciona Steam para un juego ni garantiza que el juego funcione. |
| 2026-09-27 | [Plantilla Valve `compatibilitytool.vdf`](https://github.com/ValveSoftware/Proton/blob/proton_11.0/compatibilitytool.vdf.template) | La metadata agrupa `compat_tools` y puede declarar ID, `install_path`, `display_name` y listas de OS; los paths declarados pueden ser relativos o absolutos. | LXMI valida la ruta y no ejecuta ni modifica contenido. Una plantilla no garantiza que toda versión de Proton use el mismo layout. |
| 2026-09-27 | [Documentación Valve Steam Runtime para reportar fallos](https://github.com/ValveSoftware/steam-runtime/blob/master/doc/reporting-steamlinuxruntime-bugs.md) | Steam Linux Runtime publica metadata propia y `VERSIONS.txt`; no debe clasificarse como Proton solo por el nombre del directorio. | LXMI reconoce capas conocidas de runtime y muestra versión si puede leer la fila `depot`; la presencia no demuestra uso por Wuthering Waves. |
| 2026-09-27 | Observación local de solo lectura con Steam Scanner y Proton Scanner | Se detectó Wuthering Waves (AppID `3513350`; directorio presente; compatdata ausente), Proton Experimental y tres Steam Linux Runtime. Un symlink `Steam.dll` se omitió y reportó. | Evidencia de discovery en esta instalación, no de la ventana visual, selección de Proton, lanzamiento, compatibilidad, salud de prefix o funcionamiento de XXMI. No se encontró una instalación custom para validar contra Steam real; esa variante se probó con fixtures. |
| 2026-09-27 | Observación local de solo lectura con la aplicación Tauri y Runtime Planner 0.4 | Wuthering Waves presente; compatdata/pfx ausentes; Proton Experimental y Proton Hotfix como dos candidates; tres Steam Linux Runtime; selección desconocida; readiness `NeedsInitialization`; discovery completo. El symlink conocido `Steam.dll` se reporta como informativo sin seguirlo. | Confirma solo las rutas y metadata visibles en este equipo. No indica qué herramienta seleccionaría Steam, ni salud del prefix, compatibilidad del juego o funcionamiento de XXMI. No se detectó tool custom local. |

## Hipótesis técnicas por validar

| ID | Hipótesis | Evidencia necesaria | Estado |
|---|---|---|---|
| H-01 | Existe un recorrido viable para administrar una versión compatible de XXMI/WWMI mediante Wine o Proton en Ubuntu | Documentación upstream vigente y prueba aislada con versiones registradas | Pendiente |
| H-02 | Se puede detectar de forma segura Steam, una instalación de juego y su prefix | Fuentes oficiales/formato de configuración y prueba con fixture; después, entorno de desarrollo autorizado | Pendiente |
| H-03 | Los mods objetivo pueden instalarse y activarse sin editar sus originales | Estructuras de mods autorizadas y prueba reproducible de instalación reversible | Pendiente |
| H-04 | Un cambio live es posible sin reiniciar el juego en los runtimes objetivo | Documentación o experimento autorizado, con condiciones y límites | Pendiente |
| H-05 | Los usuarios objetivo necesitan una interfaz nativa para estas tareas | Entrevistas o evidencia pública verificable; no se presume demanda comercial | Pendiente |

## Preguntas de investigación

- ¿Qué repositorio y documentación son upstream oficiales de XXMI, WWMI y ZZMI?
- ¿Qué licencias aplican a cada componente y a sus dependencias? ¿Qué obligaciones tendría una distribución o fork?
- ¿Qué versiones, juegos, plataformas y launchers están admitidos actualmente por cada runtime?
- ¿Qué limitaciones de Wine/Proton, Steam, actualizadores del juego o medidas anti-cheat afectan a la integración?
- ¿Qué formato de instalación y metadatos emplean los mods autorizados?
- ¿Hay mecanismos documentados de recarga o variables live? ¿Qué no está soportado?
- ¿Qué distribución Linux y GPU están disponibles para pruebas reproducibles?

## Regla de evidencia

Separar en futuras actualizaciones **fuente primaria**, **observación reproducida**, **inferencia** e **hipótesis**. No afirmar que un juego funciona ni que una función live es posible hasta registrar entorno, versiones, pasos y resultado. No eludir anti-cheat ni restricciones del juego.
