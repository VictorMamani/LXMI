# Guía de desarrollo

## Entorno

- Ubuntu 24.04 o posterior.
- Rust stable con rustfmt y clippy, gestionado con rustup.
- Node.js 22.13 o posterior y npm.
- Dependencias nativas de Tauri 2 para Linux: WebKitGTK 4.1, GTK, OpenSSL, librsvg y herramientas de compilación. Consultar la [guía oficial](https://v2.tauri.app/start/prerequisites/).
- Sesión gráfica X11 o Wayland para abrir la ventana desktop.

LXMI no requiere root para ejecutarse. La instalación de paquetes del sistema no se realizó desde este entorno.

## Instalar y abrir

Desde SDLC/05_Construccion/aplicacion:

```bash
cd apps/lxmi-desktop
npm ci
npm run tauri:dev
```

El botón **Scan Steam** lee roots/bibliotecas, `appmanifest_*.acf` y rutas `compatdata/<AppID>/pfx`, sin modificar archivos. Wuthering Waves se identifica por el AppID oficial `3513350`; un `pfx` se presenta únicamente como candidato, no como evidencia de Proton activo.

## Validaciones

Desde SDLC/05_Construccion/aplicacion:

```bash
cargo fmt --all -- --check
cargo clippy -p lxmi-core -p lxmi-steam --all-targets -- -D warnings
cargo test -p lxmi-core -p lxmi-steam
```

Con las dependencias nativas de Tauri instaladas, ejecutar también `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` y `cargo test --workspace`.

Desde apps/lxmi-desktop:

```bash
npm run typecheck
npm run lint
npm run format:check
npm run build
```

Las pruebas de detección crean fixtures temporales; no examinan ni modifican la instalación personal de Steam.

## Estado verificado en el entorno de trabajo

TypeScript, ESLint, Prettier, build Vite, 39 tests de lxmi-core/lxmi-steam, Clippy de esos crates y cargo fmt pasaron. `cargo check --offline --locked -p lxmi-desktop` se detuvo porque faltan libsoup-3.0 y javascriptcoregtk-4.1; por eso todavía no se pudo compilar el adapter Tauri ni abrir la ventana en este entorno.
