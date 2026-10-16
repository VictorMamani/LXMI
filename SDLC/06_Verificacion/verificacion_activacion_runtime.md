# Verificación — runtimes, selección reversible y diseño

Fecha local: 2026-10-14. Versión 0.8.7. Continúa el gestor local sin cerrar el experimento de prefix 0.8.8. No se ejecutó un juego, Proton/Wine, helper o loader en este incremento. No hubo commit ni push.

## Calidad y dominio

Pasaron fmt, check workspace all-targets, Clippy workspace all-targets con warnings denegados y tests workspace: **393 passed, 0 failed, 12 ignored**. Las tres pruebas nuevas cubren ensamblado WWMI contra payload real esperado y rechazo del bootstrap como sustituto, compatibilidad de manifiestos antiguos, activación/resolución, idempotencia, undo/redo, planes/revisiones atrasados, base incorrecta y runtime derivado alterado. Son fixtures sintéticos; su metadata de autenticidad de prueba no equivale a una firma upstream verificada.

Pasaron frontend typecheck, lint, format:check y build. El ejemplo opt-in posterior `local_profile_review` compiló y pasó Clippy all-targets del crate, sin cambios posteriores a las funciones de producción verificadas. No se añadieron dependencias.

## WWMI oficial y Tauri nativo

Se descargaron mediante el proveedor real WWMI v1.1.0 y XXMI Libraries v1.2.2. Las firmas y hashes upstream se verificaron antes del import. Se ensamblaron **32 archivos** fuera de Steam/juegos/prefixes. Fuente fija del importer: [SpectrumQT/WWMI-Package](https://github.com/SpectrumQT/WWMI-Package/releases/tag/v1.1.0); configuración contrastada con [WWMI en XXMI Launcher v2.2.1](https://github.com/SpectrumQT/XXMI-Launcher/blob/v2.2.1/src/xxmi_launcher/core/packages/model_importers/wwmi_package.py). No fue necesario incorporar un assembler de terceros: se amplió el assembler privado existente.

El harness GTK/WebKit usó el builder/registro de comandos de producción, paquetes oficiales copiados a un XDG temporal nuevo y eventos DOM dentro de su propia ventana. No sustituyó IPC ni envió teclas al escritorio. Pasaron importaciones/perfiles/persistencia y una fase adicional: preparar WWMI, revisar/aplicar perfil, comprobar que el resolver devuelve el derivado, revertir a base y rehacer reutilizando la selección. Los mods de ese recorrido fueron sintéticos.

Reproducción desde `aplicacion`, con Vite en 1420 y un store de paquetes previamente autenticados:

```bash
LXMI_NATIVE_MOD_SMOKE_SEED=/ruta/absoluta/al/lxmi bash tools/test-mod-manager-native.sh
```

Solo se copian paquetes al nuevo entorno; los runtimes se ensamblan de nuevo. Evidencia: [activacion_runtime.json](evidencias/activacion_runtime.json). Logs originales de esta sesión: `/tmp/lxmi-runtime-activation-tests.log`, `/tmp/lxmi-activation-native.log` y `/tmp/lxmi-mod-native-smoke-S2kUo8/`. El primer ensamblado oficial precedió al ajuste de textos explicativos WWMI; la fase nativa ensambló con el código corregido. Se conserva esa distinción en el JSON.

## Mods reales ZZZ del usuario

Se revisó exclusivamente `Downloads/mods-zzz`, sin ejecutar archivos del mod. RAR se leyó con `unrar` hacia stdout, con rutas/tamaños acotados, y se copió a un árbol temporal. Miyabi aporta 36 archivos y RabbitFX 5. La revisión acotada de recursos y balance condicional encontró siete `endif` huérfanos en `Miyabi Patches.ini`: se renombró a `.ini.disabled` únicamente en la copia de prueba. El INI principal de Miyabi y los INI de RabbitFX no mostraron esos defectos; eso no valida hashes del juego ni semántica completa.

Hide UID quedó fuera por comparación vacía y condicionales sin cierre. Censor Remover quedó fuera: contiene ShaderFixes sin INI y requiere una ruta de instalación que este gestor todavía no implementa. No se alteraron esos originales.

Primero se importó/aplicó el perfil en un store aislado con ZZMI v1.5.0 y Libraries v1.1.7 autenticados previamente. Después se incorporó a la biblioteca habitual `$XDG_DATA_HOME/lxmi` y se seleccionó **ZZZ — Miyabi y RabbitFX**. El runtime base existente fue verificado sin reescritura. El derivado contiene **90 archivos / 135700453 bytes**, con **2 mods**, y el resolver devolvió su ID. Antes no había perfiles ni selección para ZZZ; después la revisión de selección es 1. Se revalidaron todos los hashes de los originales tras el import: coinciden.

Evidencia durable: [mods_zzz_downloads.json](evidencias/mods_zzz_downloads.json), con inventarios resumidos, defectos, ajuste de la copia, plan, selección previa/posterior e ID resuelto. Los paquetes/copias reales permanecen privados en LXMI. El ejemplo `local_profile_review` usa la misma API de dominio; no es una prueba nativa de UI con esos mods reales.

## Continuación nativa con los mods reales

El ejemplo opt-in `selected_profile_native_review` abrió GTK/WebKit con el store habitual, navegó por DOM a ZZZ/Compatibilidad y llamó los comandos reales de producción. Confirmó que la UI restaura el perfil y sus IDs de paquetes, y que el resolver devuelve el runtime derivado de dos mods. No importó ni cambió la selección, no envió pulsaciones al escritorio y no ejecutó juegos/helpers. Resultado: **PASS**, [mods_zzz_nativo.json](evidencias/mods_zzz_nativo.json). El ejemplo y su script pasaron fmt/Clippy y comprobación de sintaxis.

La lectura actual de readiness confirmó el ejecutable en `/media/university/Steam/SteamLibrary/steamapps/common/Zenless Zone Zero/games/ZenlessZoneZero Game/ZenlessZoneZero.exe`, pero **compatdata y pfx ausentes** en esa biblioteca. Tampoco existen actualmente en las dos rutas Steam habituales del home inspeccionadas. Los paquetes y el runtime administrado sí figuran verificados; la decisión continúa `not_ready` con selección Proton desconocida. Esta observación es nueva y no debe sustituirse por los datos históricos del prefix anterior: la reserva independiente 0.8.8 se vinculó a `/home/university/.local/share/Steam/steamapps/compatdata/4162040/pfx`, ruta que ahora no existe. No corresponde consumir esa revisión histórica para la instalación actual.

La próxima comprobación depende de cómo inicia el usuario ZZZ en esta instalación: Steam debe inicializar su contexto normal si es la ruta utilizada, o debe identificarse el launcher real. No se creó un pfx artificial ni se restauró el anterior.

## Diseño y límites

La interfaz utiliza el shell, tokens, componentes y SVG originales del prototipo local autorizado. Playwright probó revisión/aplicación/undo/redo mediante IPC **simulado**, únicamente para verificar comportamiento visual. Se capturó inicio en 1440×1000, 960×760 y 390×844, además de Perfiles y revisión de runtime; no hubo overflow horizontal y la acción móvil dejó de recortarse. El detector único devolvió `[]`. Capturas y revisión independiente en `.impeccable/review/`; PRODUCT.md y DESIGN.md distinguen estas capturas de la prueba Tauri real.

**Pendiente:** ejecutar y observar una carga real en ZZZ con contexto Proton y estrategia de loader validados. `launch_with_mods_available` permanece false; no hay executor de carga para el juego. El nuevo botón de Steam sin mods está implementado, pero no se invocó contra el juego durante estas pruebas. El cambio sin recarga, conflictos semánticos e instalación de ShaderFixes siguen pendientes. La reserva/review independiente 0.8.8 conserva sus archivos; no se consumió consentimiento.

## Continuación — inicio manual en biblioteca externa

El usuario confirmó que trasladó sus juegos al otro disco e inició ZZZ desde Steam, describiendo launcher antes del juego. La observación pasiva con el parser de producción aceptó un único proceso real del juego, AppID 4162040 verificado y **Proton Experimental 1791480010 experimental-11.0-20261008-x86_64**. Compatdata observado coincide con `/media/university/Steam/SteamLibrary/steamapps/compatdata/4162040`; WINEPREFIX observado apunta a su `pfx`. Una comprobación posterior confirmó ambos directorios presentes, sin symlink en pfx. El hallazgo anterior de ausencia corresponde al estado previo al inicio manual.

Hubo permiso denegado para al menos un miembro de la ascendencia: no se afirma lectura completa del contexto. Se observó un wineserver con el mismo WINEPREFIX, lo cual tampoco prueba que el juego esté conectado a esa instancia. La selección configurada permanece separada del runtime observado en esta ejecución.

[Evidencia resumida sin PIDs, snapshots completos ni entorno crudo](evidencias/observacion_zzz_biblioteca_externa.json). El ejemplo opt-in `crates/lxmi-runtime/examples/observe_zzz.rs` solo consulta procfs con allowlist y una ventana acotada; no inicia ni modifica juegos, loaders, helpers o prefixes. No se ejecutó la reserva histórica 0.8.8, vinculada a otro prefix. La carga real de mods y la estrategia de loader siguen pendientes.

El flujo launcher primero/juego después también está contemplado en la [documentación upstream de XXMI](https://github.com/SpectrumQT/XXMI-Launcher/wiki/More-Start-Options), que distingue el inicio normal de Steam del inicio directo del ejecutable. Sus ejemplos de opciones son para Windows; no se copiaron a Steam Linux. El observer de LXMI comprueba el ejecutable del juego y su contexto, no confunde la presencia del launcher con ZZZ activo.
