# Estrategia de verificación

LXMI-0.1 a 0.5.1 validan con fixtures y árboles de directorios temporales: parser KeyValues, Steam libraries, manifests ACF, catálogo de AppIDs, compatdata/pfx, Proton/SLR classification, runtime planner, mapping WWMI/ZZMI, estructura de paquetes, dependencias, rutas y package import/assessment/planning. Los tests estándar no dependen de Steam o ZZZ reales. Un test host-only `#[ignore]` permite validar Steam/ZZZ local en modo read-only y se ejecutó por separado; eso no verifica el click de la UI, el lanzamiento del juego o compatibilidad Steam/Linux/Proton. No se lanzaron Proton, Wine ni ZZZ, ni se importó una release ZZMI real.

## Capas previstas

1. Unitarias: parser VDF, resolución de rutas, normalización/deduplicación y estados de detección.
2. Integración: detección Steam con instalaciones sintéticas temporales y errores de archivo/ruta.
3. Seguridad: traversal, rutas absolutas, enlaces, duplicados, archivos enormes, colisiones, interrupción y rollback.
4. Sistema: versiones concretas de Linux/Steam/Proton/runtime, solo después de aprobar factibilidad. El scan local 0.4 no equivale a validar selección por juego ni compatibilidad.
5. UI: typecheck, lint, errores tipados, estado de escaneo y revisión visual básica en escritorio; falta una auditoría de accesibilidad completa.

No se deben reportar como probados pasos de integración real si solo se ejecutaron fixtures. El scan de filesystem real no equivale a verificación de selección o lanzamiento. En 0.5.1, el layout upstream de ZZMI está contrastado, pero Steam y Linux/Proton siguen sin verificar; SHA-256 local no autentica el origen de un paquete.
