# Estructura actual del código

LXMI-0.6 conserva el workspace y agrega un bridge de diagnóstico independiente para probar un helper Windows bajo un Proton elegido explícitamente. Escaneos y planes siguen siendo transitorios; paquetes fuente, ensamblado ZZMI y helper staged viven bajo storage XDG. No crear módulos futuros hasta que tengan lógica real.

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
| `crates/lxmi-xxmi/src/assembly.rs` | Planifica/ensambla paquetes oficiales ZZMI + XXMI Libraries bajo XDG; configuración derivada y manifest LXMI externo al payload |
| `crates/lxmi-xxmi/src/topology.rs` | Inspecciona `pfx/dosdevices`, traduce rutas Linux/Windows y genera `LaunchTopologyPlan` read-only |
| `crates/lxmi-xxmi/src/mapping.rs` | Conserva el dry-run 0.5.2 únicamente como comparación histórica; no define destino activo |
| `crates/lxmi-xxmi/` | Modelo XXMI/WWMI/ZZMI, provenance, validación de paquete, SHA-256, importación y storage gestionado, assembly y topología declarativa; sin executor ni apply |
| `crates/lxmi-bridge/` | Servicio de bridge: selección explícita de runtime de prueba, confinamiento a storage LXMI, proceso Proton acotado, timeout, captura y resultado tipado |
| `crates/lxmi-bridge-protocol/` | Tipos serde del protocolo stdin/stdout JSON v1, nonce y respuesta del helper |
| `tools/lxmi-bridge-helper/` | Helper Windows de solo lectura: inspecciona el marker de runtime bajo el root LXMI; no enumera ni abre otros procesos |
| `Cargo.toml` | Workspace Rust |
| `apps/lxmi-desktop/src-tauri/Cargo.toml` | Paquete Rust de Tauri dentro del workspace |

El bridge se probó desde la sección Advanced con Proton Experimental elegido explícitamente y aceptación de sus posibles efectos. El helper Windows PE32+ se enlazó con MinGW, se staged bajo XDG y su hash coincide con el sidecar. El host handshake v1 y la lectura/hash del marker pasaron; el negativo de path inexistente devolvió error estructurado. Más adelante se pueden extraer `lxmi-games`, `lxmi-mods`, `xxmi-ini`, `lxmi-live` y `lxmi-gamebanana` cuando aparezca una implementación que lo requiera. No existe aún un instalador en el juego: `InstallationPlan` es un resultado efímero revisable.

Evitar crear crates vacías solo para replicar el diagrama. Empezar con los límites que un prototipo concreto necesite y extraer módulos cuando haya responsabilidades distintas.
