# Estrategia de verificación

LXMI-0.1/0.2 validan con fixtures y árboles de directorios temporales: parser KeyValues, Steam libraries, manifests ACF, catálogo de AppIDs y estados compatdata/pfx. No recorren ni alteran instalaciones reales durante los tests. La integración con un juego real requiere compatibilidad y políticas revisadas, y un entorno autorizado.

## Capas previstas

1. Unitarias: parser VDF, resolución de rutas, normalización/deduplicación y estados de detección.
2. Integración: detección Steam con instalaciones sintéticas temporales y errores de archivo/ruta.
3. Seguridad: traversal, rutas absolutas, enlaces, duplicados, archivos enormes, colisiones, interrupción y rollback.
4. Sistema: versiones concretas de Linux/Steam/Proton/runtime, solo después de aprobar factibilidad.
5. UI: typecheck, lint, errores tipados, estado de escaneo y accesibilidad básica.

No se deben reportar como probados los pasos de integración real si solo se ejecutaron fixtures.
