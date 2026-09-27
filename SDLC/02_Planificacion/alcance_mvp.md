# Alcance inicial

Este alcance es una propuesta por etapas. El primer hito no modifica instalaciones: valida detección y presenta un diagnóstico. Las acciones de escritura se habilitarán solo después de revisar fuentes, seguridad y compatibilidad.

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

## Siguiente incremento: LXMI-0.3 — Proton Runtime Discovery

Detectar instalaciones/versiones de Proton y relacionarlas de forma explicable con el juego. Mantener prefixes sin modificar y no lanzar procesos.

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
