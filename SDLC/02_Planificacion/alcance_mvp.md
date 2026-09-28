# Alcance inicial

Este alcance es una propuesta por etapas. LXMI-0.1 a 0.4 inspeccionan y planifican sin modificar Steam ni instalaciones. LXMI-0.5 a 0.5.2 importa y autentica paquetes en almacenamiento privado; LXMI-0.5.3 ensambla el runtime privado ZZMI y describe una topología, sin modificar Steam, juego o prefix. Ejecutar o aplicar cambios queda fuera hasta validar el bridge y la compatibilidad de plataforma.

## Incremento LXMI-0.1 — Aplicación y detección de Steam

- Inicializar Tauri 2, React, TypeScript y Rust en `05_Construccion/aplicacion/`.
- Mostrar sistema operativo, arquitectura, home y directorios XDG relevantes.
- Detectar Steam en rutas Linux comunes configurables/extensibles.
- Leer y parsear `steamapps/libraryfolders.vdf`; validar y deduplicar bibliotecas existentes.
- Devolver estados distinguibles para Steam ausente, configuración ausente/inválida, bibliotecas ausentes y permisos.
- Mantener el escaneo en solo lectura; no detectar juegos en este incremento.
- Añadir pruebas de parser y detección con archivos/directorios temporales, nunca con la instalación real.

## Incremento LXMI-0.2 — Juegos Steam y compatdata

- Reutilizar el parser KeyValues de `libraryfolders.vdf` para leer `appmanifest_*.acf` con límites de tamaño/profundidad.
- Enumerar manifests válidos en bibliotecas ya detectadas; un archivo inválido no aborta el resto del escaneo.
- Identificar solo Wuthering Waves por AppID `3513350`, verificado en la ficha oficial de Steam; ignorar aplicaciones desconocidas del catálogo con log.
- Construir la ruta esperada `steamapps/common/<installdir>` y distinguir carpeta presente, ausente e inválida.
- Inspeccionar, sin escribir, `steamapps/compatdata/<AppID>/` y su subdirectorio `pfx/`; presentar este último como candidato a prefix, sin inferir versión/uso/salud de Proton.
- Añadir fixtures y pruebas sintéticas de manifests, rutas y compatdata. No examinar Steam real durante tests.

## Incremento LXMI-0.3 — Proton Runtime Discovery

- Descubrir herramientas instaladas dentro de `steamapps/common` y tools custom bajo `compatibilitytools.d`.
- Reutilizar parser KeyValues para `compatibilitytool.vdf` y `toolmanifest.vdf`; distinguir Proton, Steam Linux Runtime y tipos no identificados por metadata estructural.
- Obtener version desde `version` o `VERSIONS.txt` cuando esté disponible; versión ausente no invalida por sí sola un Proton completo.
- Validar `proton` como archivo regular sin ejecutarlo; rechazar symlinks y rutas inseguras.
- Mostrar en UI tipo, source, versión, ID, rutas, status e incidencias. El Proton seleccionado por juego queda “No determinado”.
- Mantener Steam, juegos, compatdata y prefixes en solo lectura; no crear directorios.
- Verificar con fixtures y escaneo local, sin inferir lanzamiento o compatibilidad.

## Incremento LXMI-0.4 — Runtime Planning & Launch Readiness

Describir una planificación de runtime por juego a partir de evidencias disponibles, sin leer formatos internos frágiles por defecto, ejecutar Proton/Wine/juegos ni modificar launch options. La selección efectiva por juego requiere evidencia fiable y un diseño separado.

## Incremento LXMI-0.5 — XXMI / WWMI Integration Foundation

- Modelar XXMI Libraries y las integraciones específicas, comenzando con WWMI.
- Inspeccionar ubicaciones documentadas sin ejecutar archivos ni escribir en Steam/juego/prefix.
- Importar solo directorios locales a staging/storage administrado; validar estructura, rutas y hashes con límites explícitos.
- Producir un assessment y un plan revisable, sin executor ni apply.
- Mantener autenticidad, compatibilidad de lanzamiento y política de redistribución como desconocidas hasta verificarlas.

## Siguiente incremento: LXMI-0.6 — Proton Launch Topology Experiment

Resolver en un entorno autorizado, sin inyección ni mutaciones, la identidad del loader, el helper Windows, visibilidad de la ruta administrada en el prefix y la exigencia de compartir prefix. Mantener la selección Proton y compatibilidad desconocidas donde no exista evidencia. No habilitar `apply` ni escribir al juego/Steam/prefix en ese incremento.

## Hito 0 — Factibilidad

- Identificar upstream, licencias y requisitos de XXMI y runtimes candidatos.
- Definir un entorno y datos de prueba aislados.
- Confirmar qué puede detectar la aplicación sin permisos de administrador.
- Resolver si el flujo candidato puede probarse sin riesgos para cuentas o instalaciones reales.

## Hito 1 — Inspección local de solo lectura

- Registrar explícitamente las ubicaciones Steam seleccionadas por el usuario.
- Buscar instalaciones de juego configuradas manualmente o descubiertas en ubicaciones admitidas.
- Presentar ruta, ejecutable/prefix cuando se pueda verificar y nivel de confianza de la detección.
- Permitir corregir o quitar una detección guardada.

## Hito 2 — Biblioteca local y perfiles

Condicionado a la revisión de formatos y seguridad:

- importar un archivo de prueba autorizado;
- validar y extraer a un directorio temporal seguro;
- guardar originales en biblioteca gestionada;
- activar/desactivar mediante una estrategia reversible y compatible con el runtime;
- guardar perfiles por juego.

## Posibles incrementos posteriores

Configurar runtimes; administrar backups; lanzar procesos con Proton; parser INI; análisis de conflictos; controlador live; IPC; integración con GameBanana; paquetes AppImage/Flatpak y SteamOS. Cada función necesita un criterio de viabilidad antes de entrar al MVP.

## Fuera del alcance inicial

- reimplementar 3DMigoto/XXMI o modificar un fork;
- evadir anti-cheat, DRM o controles del juego;
- descargar mods automáticamente;
- instalar en juegos sin revisión de compatibilidad y permisos;
- sincronización en nube, tienda propia o repositorio de mods;
- soporte general para todas las distribuciones, launchers y juegos;
- afirmar compatibilidad SteamOS, Vulkan o “sin reinicio” antes de probarla.
