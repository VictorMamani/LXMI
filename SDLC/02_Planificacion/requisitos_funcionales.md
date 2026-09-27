# Requisitos funcionales iniciales

Estados: **implementado en código** no implica prueba con Steam real; **propuesto** = aún no implementado; **por validar** = su viabilidad o necesidad no está comprobada.

## Instalaciones y runtimes

| ID | Requisito | Estado |
|---|---|---|
| RF-00 | Mostrar OS, arquitectura, home y directorios XDG relevantes sin requerir privilegios | Implementado en código; typecheck/build web pasaron; ejecución nativa pendiente |
| RF-00.1 | Buscar Steam en rutas Linux conocidas, resolver aliases y listar bibliotecas Steam existentes | Implementado; parser/detector pasaron pruebas con fixtures; Steam real pendiente |
| RF-00.2 | Informar Steam ausente, configuración inexistente/inválida, biblioteca ausente o permiso denegado con códigos distinguibles | Implementado en código; estados principales con fixtures; denegación real de permisos pendiente |
| RF-00.3 | Mostrar Wuthering Waves como no escaneado en LXMI-0.1 | Implementado en LXMI-0.1; reemplazado por descubrimiento en LXMI-0.2 |
| RF-00.4 | Leer `appmanifest_*.acf`, validar AppID/nombre/installdir y continuar ante un manifest inválido | Implementado en `lxmi-steam`; probado con fixtures; adapter nativo Tauri pendiente de compilación |
| RF-00.5 | Enumerar manifests válidos y reconocer Wuthering Waves por AppID del registro | Implementado con AppID `3513350`; fuente oficial registrada; sin validación Steam real |
| RF-00.6 | Inspeccionar `compatdata/<AppID>/pfx` y distinguir ausente/encontrado/inválido sin seguir symlinks | Implementado en modo de solo lectura; probado con fixtures |
| RF-01 | Registrar y mostrar ubicaciones Steam elegidas por la persona usuaria | Propuesto |
| RF-02 | Detectar juegos/runtimes únicamente en ubicaciones y formatos documentados | Por validar |
| RF-03 | Permitir añadir, editar y quitar una instalación manualmente | Propuesto |
| RF-04 | Mostrar rutas detectadas, versión y confianza/estado de cada dato | Propuesto |
| RF-05 | Detectar Proton/Wine y prefix conforme a fuentes vigentes | Por validar |
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
