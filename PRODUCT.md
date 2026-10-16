# LXMI

LXMI es un gestor de mods para Linux de Wuthering Waves y Zenless Zone Zero, inspirado funcionalmente en XXMI y NoReloadManager. El objetivo del usuario es completar una aplicación de escritorio operativa; la prioridad actual es ZZZ con mods propios. No hay una definición comercial ni investigación de audiencias confirmadas.

La aplicación operativa está en `SDLC/05_Construccion/aplicacion/apps/lxmi-desktop`: React sobre Tauri/Rust. La interfaz actual, aceptada por el usuario, tiene estilo launcher: ilustraciones originales a pantalla completa por juego, barra superior con navegación y un panel de inicio con perfil, contador de mods y acción principal. Conserva los controles y fundamentos oscuros del prototipo inicial. El sistema extraído y su evolución están en [DESIGN.md](DESIGN.md).

## Trabajo que resuelve

- Importar carpetas y ZIP a una biblioteca privada con inventario y SHA-256.
- Seleccionar mods, guardar perfiles y snapshots, y recuperar esas selecciones entre sesiones.
- Preparar runtimes privados inmutables desde paquetes oficiales autenticados WWMI y ZZMI.
- Revisar perfil, volumen, ruta y argumentos antes de aplicar una selección al runtime; deshacer y rehacer mediante comprobación CAS.
- Consultar instalación, paquetes y diagnósticos desde el mismo workspace.
- Solicitar a Steam el inicio del juego mediante `steam -applaunch`, con la acción explícita «Jugar sin mods».
- Iniciar ZZZ con el perfil aplicado mediante el Hook oficial de XXMI, una copia privada de arranque y el contexto Steam de HoYoPlay.
- Importar paquetes de ShaderFixes y crear copias reparadas de mods mediante la herramienta local de reparación.

Las copias, perfiles y runtimes se preparan fuera de la instalación del juego. La revisión previa y la reversión forman parte de la operación, y los estados deben describir lo que se ha comprobado.

## Estado y límites comprobados

La biblioteca, perfiles, persistencia tras reinicio y la revisión/aplicación/deshacer/rehacer WWMI pasaron un harness nativo. Los ensambladores WWMI y ZZMI están implementados con autenticación de sus paquetes oficiales. El workspace pasó 413 pruebas, con 13 optativas ignoradas, además de formato, Clippy y las comprobaciones del frontend.

El 2026-10-15 el usuario confirmó la carga y funcionamiento de ZZZ con **ZZZ — 12 mods (4 reparados)**. El registro de ese inicio muestra ZZMI presente, hook retirado y proceso vivo tras la observación de 15 segundos; LXMI alcanzó `Running` con mods cargados. La validación corresponde a Steam, Proton y este equipo. Wuthering Waves, estabilidad prolongada y recarga sin reiniciar siguen pendientes. [Prueba confirmada](SDLC/06_Verificacion/prueba_manual_zzz_coleccion_reparada.md).

## Evidencia y continuidad

La revisión visual está en `.impeccable/review/finish-activation-review.md`. Las capturas `activation-home-{1440,960,390}-final.png`, `activation-profiles-desktop.png` y `activation-review-{desktop,mobile}.png` de ese directorio usan estados sintéticos declarados para revisar composición y reflujo. Son evidencia visual de navegador y se distinguen del harness nativo descrito arriba.

Las capturas del prototipo y sus adaptaciones registran etapas anteriores. La interfaz launcher actual dispone de una revisión nativa documentada en `SDLC/06_Verificacion/evidencias/interfaz_launcher_nativo_20261015.json`. El siguiente trabajo funcional es observar estabilidad prolongada y explorar la recarga sin reiniciar; la carga de Wuthering Waves requiere su propia validación.
