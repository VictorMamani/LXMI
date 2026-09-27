# Decisiones técnicas iniciales

Todas las decisiones son provisionales hasta terminar el estudio de factibilidad.

| ID | Propuesta | Motivo | Revisión necesaria |
|---|---|---|---|
| ADR-001 | Mantener LXMI como un proyecto con Runtime Manager y Mod Manager | Comparten juegos, rutas, perfiles, logs y configuración | Reconsiderar si la independencia de publicación o licencia lo exige |
| ADR-002 | Explorar Tauri, React/TypeScript y Rust | Propuesta del usuario; interfaz web embebida con core nativo | Confirmar versión actual, APIs, empaquetado y dependencias Linux antes de iniciar |
| ADR-003 | Explorar SQLite para metadatos locales | Adecuado como candidato para datos de escritorio locales | No guardar dentro de biblioteca de mods; revisar backups/migraciones |
| ADR-004 | No reimplementar XXMI en el primer MVP | Reducir alcance; administrar configuración si las licencias y compatibilidad lo permiten | Verificar repositorios upstream y términos |
| ADR-005 | Preservar originales y generar copias gestionadas | Facilita reversión y protege los archivos fuente | Comprobar espacio, permisos y comportamiento del runtime |
| ADR-006 | Posponer GameBanana, SteamOS y bridge IPC | Evitar dependencias antes de validar el caso local básico | Revisar API/términos, documentación y entorno al llegar a esos hitos |
| ADR-007 | No crear otro repositorio de runtime por ahora | Todavía no hay fork ni componente derivado | Revaluar solo con evidencia de necesidad y revisión de licencia |
| ADR-008 | Tauri 2 + React/TypeScript con Vite para LXMI-0.1/0.2 | UI de escritorio solicitada; Vite sirve una SPA local sin SSR | El frontend pasa validaciones; falta compilar/abrir ventana nativa por deps WebKitGTK/libsoup |
| ADR-009 | Mantener `apps/lxmi-desktop`, `crates/lxmi-core` y `crates/lxmi-steam`; añadir módulos internos según responsabilidad | La UI, modelos y discovery Steam/juegos son límites suficientes por ahora | No crear crates futuras hasta que tengan lógica propia |
| ADR-010 | No agregar SQLite en LXMI-0.1 | El sistema y las bibliotecas se detectan en cada escaneo; no hace falta persistencia | Reconsiderar al administrar instalaciones, perfiles o metadatos |
| ADR-011 | Implementar un parser VDF acotado a KeyValues y encapsulado en `lxmi-steam` | Este incremento solo consume `libraryfolders.vdf`; fixtures permiten probar sintaxis necesaria | Si el alcance VDF crece, revisar parser existente antes de ampliar el parser propio |
| ADR-012 | Reutilizar el parser KeyValues para `libraryfolders.vdf` y `appmanifest_*.acf` | Ambos archivos tienen estructura de objetos/escalares compatible; evita parsers duplicados | Mantener límites de 2 MiB y profundidad, y ampliar solo ante fixtures nuevos |
| ADR-013 | Identificar juegos soportados mediante un registro de AppIDs Steam, no por coincidencia de nombre | Evita falsos positivos por nombres localizados/alterados; Wuthering Waves usa AppID `3513350` según Steam | La identificación de producto no demuestra instalación válida ni compatibilidad del juego |
| ADR-014 | Consultar compatdata y `pfx` con `symlink_metadata` y tratar `pfx` como candidato | La inspección debe ser pasiva y no seguir enlaces externos | LXMI-0.2 no determina versión/uso/salud de Proton; revisar casos reales solo en entorno autorizado |

Tauri requiere toolchain Rust y dependencias nativas Linux. La decisión de stack inicial está tomada; la ventana sigue pendiente de compilación y ejecución en un entorno con WebKitGTK, JavaScriptCoreGTK y libsoup.
