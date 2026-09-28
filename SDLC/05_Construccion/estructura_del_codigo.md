# Estructura actual del código

LXMI-0.5 mantiene el workspace pequeño y agrega `lxmi-xxmi` para el dominio de paquetes, importación gestionada, detección y planificación declarativa. Los escaneos/planes siguen siendo transitorios; paquetes importados y su metadata persisten en XDG, sin SQLite. No crear módulos futuros hasta que tengan lógica real.

| Ruta propuesta | Responsabilidad |
|---|---|
| `apps/lxmi-desktop/` | Shell Tauri 2, UI React/TypeScript y comandos adaptadores |
| `crates/lxmi-core/` | Información del sistema, catálogo genérico de juegos, modelos de instalación/compatdata e interfaz del detector |
| `crates/lxmi-steam/src/vdf.rs` | Lexer/parser KeyValues reutilizado para libraryfolders y appmanifests |
| `crates/lxmi-steam/src/manifest.rs` | Extracción validada de AppID, nombre e installdir |
| `crates/lxmi-steam/src/game_scanner.rs` | Enumeración de `appmanifest_*.acf`, validación de carpetas e incidencias parciales |
| `crates/lxmi-steam/src/compatdata.rs` | Inspección pasiva de compatdata/pfx, rechazando enlaces simbólicos |
| `crates/lxmi-steam/src/discovery.rs` | Orquesta Steam roots, catálogo de juegos y compatdata |
| `crates/lxmi-steam/src/scanner.rs` | Detección de Steam roots y bibliotecas de 0.1 |
| `crates/lxmi-proton/` | Discovery de compatibility tools, parser de metadata Valve, validación de rutas, classification Proton/Steam Linux Runtime y lectura acotada de versiones |
| `crates/lxmi-runtime/` | Composición de discovery, candidatos, selection, compatdata/prefix, requisitos, evidencia, issues y readiness; sin filesystem ni ejecución |
| `crates/lxmi-xxmi/` | Modelo XXMI/WWMI, validación de paquete, SHA-256, importación acotada, storage gestionado, detection y plan de instalación sin executor |
| `Cargo.toml` | Workspace Rust |
| `apps/lxmi-desktop/src-tauri/Cargo.toml` | Paquete Rust de Tauri dentro del workspace |

Más adelante se pueden extraer `lxmi-games`, `lxmi-mods`, `xxmi-ini`, `lxmi-live` y `lxmi-gamebanana` cuando aparezca una implementación que lo requiera. No existe aún un instalador en el juego: `InstallationPlan` es un resultado efímero revisable.

Evitar crear crates vacías solo para replicar el diagrama. Empezar con los límites que un prototipo concreto necesite y extraer módulos cuando haya responsabilidades distintas.
