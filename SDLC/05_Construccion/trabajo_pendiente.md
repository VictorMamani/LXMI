# Trabajo pendiente

## LXMI-0.1

- [x] Crear workspace con apps/lxmi-desktop, lxmi-core y lxmi-steam.
- [x] Añadir UI Tauri/React/TypeScript y comandos adaptadores para sistema y escaneo Steam.
- [x] Detectar raíces Steam comunes, parsear VDF y validar/deduplicar bibliotecas.
- [x] Mantener Wuthering Waves sin escanear y no tocar manifests/juegos/prefixes.
- [x] Añadir pruebas sintéticas de rutas, VDF, bibliotecas ausentes y errores.
- [x] Pasar tests/clippy de crates principales, fmt y checks/build de frontend.
- [ ] Instalar dependencias nativas de Tauri en un entorno de desarrollo autorizado y ejecutar cargo check, cargo test --workspace, cargo clippy --workspace y npm run tauri:dev.
- [ ] Recorrer el resultado visual de la ventana y comprobar el escaneo en un Steam de desarrollo, manteniendo el acceso de solo lectura.

## LXMI-0.2 — Game + compatdata Discovery

- [x] Reutilizar el parser KeyValues para `appmanifest_*.acf` y validar campos requeridos.
- [x] Enumerar manifests por biblioteca, continuar ante inválidos y validar nombre de archivo contra AppID.
- [x] Añadir catálogo central de juegos y reconocer Wuthering Waves por AppID `3513350` verificado con Steam.
- [x] Construir la ruta esperada del juego y diferenciar carpeta instalada/ausente/inválida.
- [x] Inspeccionar compatdata y `pfx` mediante operaciones de solo lectura, rechazando enlaces simbólicos.
- [x] Mostrar resultado, ruta, estado, candidato pfx y errores/avisos en el frontend.
- [x] Añadir fixtures sintéticos y pasar tests/clippy de crates y validaciones web.
- [ ] Compilar el paquete Tauri y abrir la ventana tras disponer de las bibliotecas nativas requeridas; no se validó todavía Steam real.

## Siguiente incremento: LXMI-0.3 — Proton Runtime Discovery

- [ ] Detectar instalaciones y versiones de Proton desde fuentes/documentación verificadas.
- [ ] Relacionar de forma explicable el AppID/juego con los datos de compatdata disponibles.
- [ ] No inferir runtime activo, salud de prefix o compatibilidad basándose solo en directorios.
- [ ] Mantener lectura pasiva: no ejecutar Proton/Wine ni modificar prefixes.
- [ ] Probar layouts con fixtures antes de una comprobación autorizada de entorno real.

## Estado de Git

- [ ] Resolver el límite del repositorio antes de commits: hoy Git se resuelve a `/home/university`, `master`, sin commits ni remote; `19_LXMI` no es repositorio independiente. No se ejecutó `git init`, no se cambió remote y no se publicó nada.

## Antes de integrar runtimes o contenido de mods

- [ ] Registrar repositorios upstream oficiales y leer licencias/dependencias.
- [ ] Verificar documentación y políticas aplicables del juego/runtime; no evadir anti-cheat ni controles.
- [ ] Diseñar importación segura y pruebas de traversal/enlaces/tamaño antes de extraer archivos.
- [ ] Investigar función live e IPC de forma separada; no asumir que existe un protocolo upstream.
- [ ] Evaluar SteamOS y distribución solo después de un MVP probado.
