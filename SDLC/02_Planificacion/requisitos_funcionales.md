# Requisitos funcionales iniciales

Estados: **implementado** indica que la función existe y fue verificada en el tipo de entorno mencionado; no implica compatibilidad del juego. **Propuesto** = aún no implementado; **por validar** = la viabilidad o necesidad no está comprobada.

## Instalaciones y runtimes

| ID | Requisito | Estado |
|---|---|---|
| RF-00 | Mostrar OS, arquitectura, home y directorios XDG relevantes sin requerir privilegios | Implementado; Tauri workspace compila; frontend typecheck/build pasa |
| RF-00.1 | Buscar Steam en rutas Linux conocidas, resolver aliases y listar bibliotecas Steam existentes | Implementado; fixtures y Steam local comprobados |
| RF-00.2 | Informar Steam ausente, configuración inexistente/inválida, biblioteca ausente o permiso denegado con códigos distinguibles | Implementado; estados con fixtures; denegación real de permisos pendiente |
| RF-00.3 | Mostrar Wuthering Waves como no escaneado en LXMI-0.1 | Implementado en LXMI-0.1; reemplazado por descubrimiento en LXMI-0.2 |
| RF-00.4 | Leer `appmanifest_*.acf`, validar AppID/nombre/installdir y continuar ante un manifest inválido | Implementado en `lxmi-steam`; fixtures y Steam local comprobados |
| RF-00.5 | Enumerar manifests válidos y reconocer Wuthering Waves por AppID del registro | Implementado; fixture y Steam local comprobaron AppID `3513350` y directorio presente |
| RF-00.6 | Inspeccionar `compatdata/<AppID>/pfx` y distinguir ausente/encontrado/inválido sin seguir symlinks | Implementado y probado con fixtures; Steam local mostró compatdata ausente |
| RF-00.7 | Descubrir compatibility tools en Steam Libraries y `compatibilitytools.d` leyendo metadata estructural | Implementado en `lxmi-proton`; fixtures para tools Steam/custom y scan local para tools Steam |
| RF-00.8 | Distinguir Proton, Steam Linux Runtime, otras herramientas y estado/versión disponible | Implementado; fixtures y Steam local detectaron Proton Experimental, Proton Hotfix y tres Steam Linux Runtime |
| RF-00.9 | Mostrar herramientas disponibles junto al estado de Wuthering Waves sin inferir la selección de Steam | Implementado en UI/command; typecheck/build, scan local y ventana Tauri comprobados |
| RF-00.10 | Crear un plan declarativo por juego que distinga selección observada/desconocida, candidates, compatdata/prefix, requisitos, evidencia y readiness | Implementado en `lxmi-runtime`; tests sintéticos y scan local comprobados. No ejecuta procesos ni modifica Steam |
| RF-00.11 | Modelar XXMI Libraries separado de integraciones de juego (WWMI/ZZMI/GIMI) | Implementado en `lxmi-xxmi` para Libraries y WWMI; ZZMI/GIMI son solo tipos de dominio |
| RF-00.12 | Detectar estructura WWMI en rutas candidatas finitas y reportar evidencia/incompleto/ausente | Implementado; fixtures sintéticos y Steam/juego local en read-only |
| RF-00.13 | Importar carpeta local validada a storage privado XDG con hash, staging y manifest interno separado | Implementado; fixtures y assets upstream importados en `/tmp`. Solo carpeta; archivos upstream no se ejecutan |
| RF-00.14 | Conservar versión raw, procedencia declarada, inventario, SHA-256 y verificar cambios antes de listar/planificar | Implementado. Checksums no verifican firma ni autenticidad |
| RF-00.15 | Evaluar requisitos de integración dejando compatibilidad y selección desconocidas sin evidencia | Implementado en `lxmi-xxmi`; package/runtime no implican compatibilidad |
| RF-00.16 | Generar Installation Plan con targets, hashes previos, backup requerido y warnings, sin apply | Implementado; `executable=false`; se probó que no modifica filesystem fuera del staging |
| RF-00.17 | Inspeccionar/importar/revisar plan desde UI funcional | Adaptador UI/commands implementado; typecheck/build/arranque Tauri comprobados. Clickflow de IPC en ventana nativa no recorrido |
| RF-01 | Registrar y mostrar ubicaciones Steam elegidas por la persona usuaria | Propuesto |
| RF-02 | Detectar juegos/runtimes únicamente en ubicaciones y formatos documentados | Por validar |
| RF-03 | Permitir añadir, editar y quitar una instalación manualmente | Propuesto |
| RF-04 | Mostrar rutas detectadas, versión y confianza/estado de cada dato | Propuesto |
| RF-05 | Descubrir herramientas Proton y evaluar readiness sin asumir selección ni estado saludable del prefix | Implementado para discovery y planificación; selección y salud del prefix quedan desconocidas cuando no hay evidencia |
| RF-06 | Lanzar un juego o runtime con argumentos y entorno revisados por la persona usuaria | Por validar; posterior al hito de inspección |
| RF-07 | Capturar y presentar logs útiles sin exponer información sensible | Propuesto |
| RF-08 | Respaldar y restaurar configuraciones gestionadas por LXMI | Por validar |

## Biblioteca y perfiles

| ID | Requisito | Estado |
|---|---|---|
| RF-09 | Importar un archivo local de mod compatible y autorizado | Propuesto |
| RF-10 | Validar rutas del archivo antes de extraerlo y rechazar escapes del destino | Obligatorio antes de habilitar importación |
| RF-11 | Calcular SHA-256 y señalar posibles duplicados | Propuesto |
| RF-12 | Guardar metadatos locales editables sin sobrescribir el archivo original | Propuesto |
| RF-13 | Activar y desactivar un mod usando una operación reversible | Por validar según runtime y sistema de archivos |
| RF-14 | Crear perfiles por juego y seleccionar un conjunto de mods | Propuesto |
| RF-15 | Informar conflicto potencial cuando exista evidencia verificable | Posterior; no prometer detección completa |

## Runtime live y fuentes externas

| ID | Requisito | Estado |
|---|---|---|
| RF-16 | Analizar INI compatible preservando comentarios y contenido desconocido | Investigación técnica pendiente |
| RF-17 | Generar archivos gestionados separados de los originales | Condición de seguridad propuesta |
| RF-18 | Cambiar opciones live mientras corre el juego | Hipótesis técnica; fuera del primer MVP |
| RF-19 | Comunicarse con un bridge del runtime mediante IPC local | Exploración posterior, condicionada a upstream/licencia |
| RF-20 | Buscar/descargar desde GameBanana | Fuera del MVP; revisar API y términos antes de diseñar |

Los roles son locales al equipo; no se diseña autenticación multiusuario ni sincronización remota en esta etapa.
