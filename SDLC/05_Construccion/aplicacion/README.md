# LXMI — aplicación

Incrementos LXMI-0.1/0.2 de **Tauri 2 + React + TypeScript + Rust**. El workspace contiene la app desktop y dos crates: lxmi-core y lxmi-steam. No usa SQLite ni escribe en Steam, juegos o prefixes.

## Requisitos de desarrollo

- Ubuntu con dependencias nativas de Tauri 2 instaladas. Consulta la [guía oficial de prerrequisitos Linux de Tauri](https://v2.tauri.app/start/prerequisites/); incluye WebKitGTK 4.1, GTK, OpenSSL y herramientas de compilación.
- Rust stable mediante rustup, con rustfmt y clippy.
- Node.js 22.13 o posterior y npm.
- Una sesión de escritorio Linux para abrir la ventana de Tauri.

No se necesitan credenciales, permisos root, Steam en ejecución, juegos instalados ni prefixes para compilar y probar el detector.

## Estructura

| Ruta | Responsabilidad |
|---|---|
| apps/lxmi-desktop/ | Interfaz React y paquete Tauri |
| apps/lxmi-desktop/src-tauri/ | Commands adaptadores y configuración de escritorio |
| crates/lxmi-core/ | Información del sistema, registro de juegos y modelos de instalación/compatdata |
| crates/lxmi-steam/ | Steam roots/bibliotecas, parser KeyValues, manifests, game scanner y compatdata discovery |

## Instalar dependencias y ejecutar

Desde este directorio:

```bash
cd apps/lxmi-desktop
npm ci
npm run tauri:dev
```

**Scan Steam** inspecciona roots, bibliotecas, manifests y `compatdata` en modo de solo lectura. Wuthering Waves se identifica por AppID `3513350`, verificado en la [ficha oficial de Steam](https://store.steampowered.com/app/3513350/Wuthering_Waves/). Si se encuentra `pfx`, LXMI lo muestra como candidato; no afirma que Proton esté activo ni que el juego sea compatible.

## Validaciones

Desde la raíz de esta aplicación, con los prerrequisitos nativos de Tauri instalados:

```bash
cargo fmt --all -- --check
cargo clippy -p lxmi-core -p lxmi-steam --all-targets -- -D warnings
cargo test -p lxmi-core -p lxmi-steam
```

Cuando estén instaladas las dependencias de desarrollo Linux de Tauri, se podrá verificar el workspace completo con `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` y `cargo test --workspace`.

Desde apps/lxmi-desktop/:

```bash
npm run typecheck
npm run lint
npm run format:check
npm run build
```

La suite Rust usa fixtures y directorios temporales. No examina ni modifica la instalación personal de Steam.

## Estado de LXMI-0.1/0.2

Implementa información del sistema, detección Steam, parser KeyValues reutilizado por `libraryfolders.vdf` y `appmanifest_*.acf`, detección del juego soportado, inspección pasiva de compatdata/pfx, incidencias tipadas y fixtures sintéticos. No detecta versiones de Proton/Wine, no ejecuta juegos y no instala o configura runtimes, mods o perfiles. Las pruebas no recorren Steam real.

El frontend pasa typecheck/lint/format/build. Los crates Rust `lxmi-core` y `lxmi-steam` pasan tests y Clippy. La compilación/ejecución nativa Tauri está pendiente porque en el entorno actual faltan libsoup 3, JavaScriptCoreGTK 4.1 y WebKitGTK 4.1.
