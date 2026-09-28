# Estrategia de verificación

LXMI-0.1/0.2/0.3 validan con fixtures y árboles de directorios temporales: parser KeyValues, Steam libraries, manifests ACF, catálogo de AppIDs, compatdata/pfx, compatibilitytool/tool manifests, Proton/SLR classification, rutas, versiones y symlinks. Los tests no recorren ni alteran instalaciones reales. Además se ejecutó un scan local de solo lectura y se revisó visualmente la ventana Tauri con resultados. No se lanzaron Proton, Wine ni juegos.

## Capas previstas

1. Unitarias: parser VDF, resolución de rutas, normalización/deduplicación y estados de detección.
2. Integración: detección Steam con instalaciones sintéticas temporales y errores de archivo/ruta.
3. Seguridad: traversal, rutas absolutas, enlaces, duplicados, archivos enormes, colisiones, interrupción y rollback.
4. Sistema: versiones concretas de Linux/Steam/Proton/runtime, solo después de aprobar factibilidad.
5. UI: typecheck, lint, errores tipados, estado de escaneo y revisión visual básica en escritorio; falta una auditoría de accesibilidad completa.

No se deben reportar como probados pasos de integración real si solo se ejecutaron fixtures. El scan de filesystem real no equivale a verificación de selección o lanzamiento.
