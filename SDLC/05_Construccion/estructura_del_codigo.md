# Estructura actual del código

LXMI-0.2 conserva el workspace pequeño y añade módulos internos con responsabilidades implementadas. No crear los módulos futuros hasta que tengan lógica real.

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
| `Cargo.toml` | Workspace Rust |
| `apps/lxmi-desktop/src-tauri/Cargo.toml` | Paquete Rust de Tauri dentro del workspace |

Más adelante se pueden extraer `lxmi-games`, `lxmi-proton`, `lxmi-runtime`, `lxmi-mods`, `xxmi-ini`, `lxmi-live` y `lxmi-gamebanana` cuando aparezca una implementación que lo requiera.

Evitar crear crates vacías solo para replicar el diagrama. Empezar con los límites que un prototipo concreto necesite y extraer módulos cuando haya responsabilidades distintas.
