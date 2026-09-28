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

El botón **Scan Steam** lee roots/bibliotecas, `appmanifest_*.acf`, `compatdata/<AppID>/pfx` y metadata de compatibility tools, sin modificar archivos ni ejecutar Proton/Wine. Wuthering Waves se identifica por el AppID oficial `3513350`; un `pfx` se presenta únicamente como candidato. Las tools Proton disponibles no se presentan como selección efectiva del juego.

## Validaciones

Desde SDLC/05_Construccion/aplicacion:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Desde apps/lxmi-desktop:

```bash
npm run typecheck
npm run lint
npm run format:check
npm run build
```

Las pruebas de detección crean fixtures temporales; no examinan ni modifican la instalación personal de Steam. Para un scan local bajo demanda, la aplicación solo lee los archivos y directorios descritos en el resultado; la comprobación real registrada no lanzó runtimes ni escribió en Steam.

## Estado verificado en el entorno de trabajo

En LXMI-0.3 pasaron `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`. El scan local reportó Wuthering Waves instalado, compatdata ausente, Proton Experimental y tres Steam Linux Runtime; una entrada symlink `Steam.dll` fue omitida. La ventana Tauri se inició y revisó visualmente con los resultados locales. El árbol inicial Git estaba limpio y remoto/branch se verificaron.
