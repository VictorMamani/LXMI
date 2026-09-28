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
| ADR-008 | Tauri 2 + React/TypeScript con Vite para LXMI-0.1/0.2/0.3 | UI de escritorio solicitada; Vite sirve una SPA local sin SSR | Workspace compilado; ventana iniciada y revisada visualmente con Steam local |
| ADR-009 | Mantener `apps/lxmi-desktop`, `crates/lxmi-core`, `crates/lxmi-steam` y extraer `crates/lxmi-proton` | `lxmi-proton` tiene responsabilidad propia de discovery de compatibility tools y reutiliza modelos/parser sin ciclo | No crear crates futuras hasta que tengan lógica propia |
| ADR-010 | No agregar SQLite en LXMI-0.1 | El sistema y las bibliotecas se detectan en cada escaneo; no hace falta persistencia | Reconsiderar al administrar instalaciones, perfiles o metadatos |
| ADR-011 | Implementar un parser VDF acotado a KeyValues y encapsulado en `lxmi-steam` | Este incremento solo consume `libraryfolders.vdf`; fixtures permiten probar sintaxis necesaria | Si el alcance VDF crece, revisar parser existente antes de ampliar el parser propio |
| ADR-012 | Reutilizar el parser KeyValues para `libraryfolders.vdf` y `appmanifest_*.acf` | Ambos archivos tienen estructura de objetos/escalares compatible; evita parsers duplicados | Mantener límites de 2 MiB y profundidad, y ampliar solo ante fixtures nuevos |
| ADR-013 | Identificar juegos soportados mediante un registro de AppIDs Steam, no por coincidencia de nombre | Evita falsos positivos por nombres localizados/alterados; Wuthering Waves usa AppID `3513350` según Steam | La identificación de producto no demuestra instalación válida ni compatibilidad del juego |
| ADR-014 | Consultar compatdata y `pfx` con `symlink_metadata` y tratar `pfx` como candidato | La inspección debe ser pasiva y no seguir enlaces externos | LXMI-0.2 no determina versión/uso/salud de Proton; revisar casos reales solo en entorno autorizado |
| ADR-015 | Hacer `KeyValuesValue` y `parse_key_values_document` reutilizables desde `lxmi-steam` | `compatibilitytool.vdf` y `toolmanifest.vdf` usan Valve KeyValues; se evita crear un parser duplicado o crate vacía | Los fixtures cubren estructuras actuales de metadata; extender solo con archivos/formats verificados |
| ADR-016 | Clasificar compatibility tools por metadata estructural, no por nombre de carpeta | Proton se identifica por layer `proton` y entrypoint `proton`; Steam Linux Runtime se distingue por layers conocidos y metadata `VERSIONS.txt` | Los layouts pueden evolucionar. La falta de versión no invalida Proton; no se infiere selección por juego |
| ADR-017 | Leer custom `install_path` con validación de componentes y sin seguir symlinks | Valve documenta paths relativos y absolutos; se rechaza traversal y se inspeccionan directorios existentes sin escritura | No se instala ni escribe en los destinos descubiertos; ampliar distribución Flatpak/Snap requiere análisis separado |

La decisión de stack inicial está tomada. El workspace compila en este entorno con dependencias nativas Linux disponibles; el arranque de Tauri y la revisión visual de la ventana están comprobados.
