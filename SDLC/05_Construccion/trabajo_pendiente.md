# Trabajo pendiente

## LXMI-0.1 a 0.3 — estado

- [x] Workspace Tauri 2/React/TypeScript/Rust con `lxmi-core`, `lxmi-steam` y `lxmi-proton`.
- [x] Detección Steam roots y bibliotecas, manifests, Wuthering Waves, compatdata/pfx y compatibility tools en modo de solo lectura.
- [x] Reutilizar Valve KeyValues; reconocer Proton y distinguir Steam Linux Runtime mediante metadata estructural.
- [x] Fixtures para VDF/ACF, Proton, custom compatibility metadata, SLR, estado incompleto, rutas y symlinks.
- [x] `cargo fmt`, check, Clippy con `-D warnings`, tests Rust de workspace, frontend typecheck/lint/format/build.
- [x] Escaneo real de Steam: Wuthering Waves instalado, compatdata ausente, Proton Experimental y tres Steam Linux Runtime. Symlink `Steam.dll` omitido con aviso.
- [x] El repositorio está en la ruta LXMI, branch `main`, con remote `origin` `git@github.com:VictorMamani/LXMI.git`.
- [ ] Verificar una herramienta custom instalada localmente; su formato y rutas están cubiertos por fixtures.

## Siguiente incremento: LXMI-0.4 — Runtime Selection & Launch Planning

- [ ] Diseñar un modelo de plan de runtime por juego que distinga facts observados de selección desconocida.
- [ ] Investigar evidencia fiable para conocer selección de compatibilidad en Steam, sin parsear formatos internos frágiles por defecto.
- [ ] Definir el alcance como planificación/diagnóstico; no ejecutar Proton, Wine ni juego, ni cambiar launch options/prefix.
- [ ] Agregar fixtures y criterios para herramienta disponible, compatdata ausente y selección no determinada.
- [ ] Mantener evaluación de ejecución futura separada y condicionada a pruebas/seguridad.

## Estado de Git

- Worktree modificado por LXMI-0.3; no se crearon commits y no se hizo push.
- El remoto y branch son correctos; no reorganizar el repositorio.

## Antes de integrar runtimes o contenido de mods

- [ ] Registrar repositorios upstream oficiales de XXMI/WWMI/ZZMI y revisar licencias/dependencias antes de integración.
- [ ] Verificar documentación y políticas aplicables del juego/runtime; no evadir anti-cheat ni controles.
- [ ] Diseñar importación segura y pruebas de traversal/enlaces/tamaño antes de extraer archivos.
- [ ] Investigar función live e IPC de forma separada; no asumir que existe un protocolo upstream.
- [ ] Evaluar SteamOS y distribución solo después de un MVP probado.
