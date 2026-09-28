# LXMI — aplicación

Incrementos LXMI-0.1/0.2/0.3 de **Tauri 2 + React + TypeScript + Rust**. El workspace contiene la app desktop y crates `lxmi-core`, `lxmi-steam` y `lxmi-proton`. No usa SQLite ni escribe en Steam, juegos, compatibility tools o prefixes.

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
| crates/lxmi-proton/ | Discovery de compatibility tools Steam/custom; parser Valve KeyValues reutilizado; metadata, clasificación y versión |

## Instalar dependencias y ejecutar

Desde este directorio:

```bash
cd apps/lxmi-desktop
npm ci
npm run tauri:dev
```

**Scan Steam** inspecciona roots, bibliotecas, manifests, `compatdata` y metadata de compatibility tools en modo de solo lectura. Wuthering Waves se identifica por AppID `3513350`, verificado en la [ficha oficial de Steam](https://store.steampowered.com/app/3513350/Wuthering_Waves/). Proton se valida mediante metadata estructural y un entrypoint `proton` regular; Steam Linux Runtime se clasifica aparte. Si se encuentra `pfx`, LXMI lo muestra como candidato. La app no determina qué Proton seleccionó Steam ni afirma que el juego sea compatible.

Las ubicaciones y metadata se basan en [Valve Proton](https://github.com/ValveSoftware/Proton), la [plantilla oficial `compatibilitytool.vdf`](https://github.com/ValveSoftware/Proton/blob/proton_11.0/compatibilitytool.vdf.template) y la [documentación oficial Steam Runtime](https://github.com/ValveSoftware/steam-runtime/blob/master/doc/reporting-steamlinuxruntime-bugs.md). LXMI no ejecuta esos runtimes.

## Validaciones

Desde la raíz de esta aplicación, con los prerrequisitos nativos de Tauri instalados:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Desde apps/lxmi-desktop/:

```bash
npm run typecheck
npm run lint
npm run format:check
npm run build
```

La suite Rust usa fixtures y directorios temporales. Una comprobación local separada ejecutó los scanners contra Steam en solo lectura. No modifica la instalación personal de Steam.

## Estado de LXMI-0.1/0.2/0.3

Implementa información del sistema, detección Steam, parser KeyValues reutilizado por VDF/ACF y metadata de tools, catálogo Wuthering Waves, inspección pasiva de compatdata/pfx, discovery de Proton y Steam Linux Runtime, incidencias tipadas y fixtures sintéticos. No determina runtime seleccionado por juego; no ejecuta Proton/Wine ni instala/configura runtimes, mods o perfiles.

Verificación LXMI-0.3: el workspace Tauri/Rust pasa `cargo check`, Clippy con `-D warnings`, tests y fmt. El frontend pasa typecheck/lint/format/build. El scan real encontró Wuthering Waves con compatdata ausente, Proton Experimental y tres Steam Linux Runtime; omitió una entrada symlink `Steam.dll`. La ventana Tauri se inició y revisó visualmente con la lista de runtimes y la incidencia.
