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

## Estado histórico de LXMI-0.4

En LXMI-0.4 pasaron `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace` (69 tests), `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`. El scan local read-only detectó Wuthering Waves con directorio presente, compatdata/pfx ausentes, Proton Experimental y Proton Hotfix como dos candidatos, tres Steam Linux Runtime, selección desconocida y readiness `NeedsInitialization`. La ventana Tauri inició y el scan se ejecutó desde ella. `Steam.dll` se omitió sin seguir su symlink y no hizo parcial el scan; otros symlinks candidatos siguen como advertencias. No se encontró una tool custom local. No se ejecutaron Steam, Proton, Wine ni el juego.

## Estado verificado de LXMI-0.5

Pasaron `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy con `-D warnings`, `cargo test --workspace` (114 tests), `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`. `npm run tauri:dev` inició el binario nativo y el frontend Vite respondió; el proceso fue detenido al terminar la comprobación. Importación y plan se verificaron con fixtures y carpetas de releases upstream en `/tmp`; no se ejecutó contenido ni se modificaron Steam, juego o prefix.

En el scan local del 2026-09-28 no apareció el manifest de Wuthering Waves; las carpetas Proton vistas no tenían metadata/entrypoint suficiente para clasificarse y `compatdata/3513350` no existía. El planner dejó runtime/prefix desconocidos y readiness `Blocked`. El resultado anterior de 0.4 se conserva como histórico. El flujo IPC de importación desde clics en ventana nativa queda **NO COMPROBADO**. Detalles en `../06_Verificacion/verificacion_0_5.md`.

## Estado verificado de LXMI-0.5.3

`cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` y `cargo test --workspace` pasaron; el run estándar tuvo 149 tests aprobados y 5 ignorados por requerir confirmación, red o un host con ZZZ. Pasaron `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`.

La prueba host-only opt-in validó los packages administrados ZZMI v1.5.0 y XXMI Libraries v1.1.7, ensambló 49 archivos bajo XDG y mapeó el importer por `Z:`. Selección de Proton y requisito de compartir prefix siguen desconocidos. Rutas candidatas del juego y `pfx/dosdevices` quedaron iguales antes/después. No se ejecutaron Steam, Proton, Wine, helper o ZZZ, y no se escribió en el juego ni en prefix.

Había una ventana/proceso `tauri dev` ya activo. No se cerró ni se reemplazó; tampoco se completó interacción manual de sus comandos en esta revisión. La validación funcional manual de los botones de ensamblado/topología sigue **NO COMPROBADA**. Ver `../06_Verificacion/verificacion_0_5_3.md`.

## LXMI-0.6 — validación del bridge

El helper se construyó con target `x86_64-pc-windows-gnu` y linker `x86_64-w64-mingw32-gcc` (GCC 13-win32). El artefacto PE32+ es de 1,517,755 bytes; build y staged tienen SHA-256 `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2`. No se instaló toolchain con sudo.

El host bridge positivo y negativo se probaron mediante Proton Experimental seleccionado explícitamente y solo con el prefix LXMI aislado. Pasaron nonce/protocolo, path Windows real, lectura y hash de `runtime-manifest.json`, exit 0; el caso no encontrado respondió `runtime_not_visible`, exit 2. La ventana Tauri nativa recorrió scan, helper/runtime ausentes y resultado exitoso. GameRuntimePlan.selection quedó `unknown`.

Pasaron `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy `-D warnings`, `cargo test --workspace` (166 pasados, 6 ignorados), typecheck, ESLint, Prettier y build. Proton escribió dentro del prefix/contexto de test LXMI y tocó el timestamp de su `dist.lock`; no se alteró ZZZ ni `compatdata/4162040`. Ver `../06_Verificacion/verificacion_0_6.md` para la evidencia y límites completos.
