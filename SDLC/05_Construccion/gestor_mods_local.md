# Gestor local de mods y perfiles

Implementado el 2026-10-14 por petición del usuario, quien priorizó el gestor. Es una línea de trabajo independiente del experimento de prefix 0.8.8. La versión sigue en 0.8.7; no se abrió otra sesión de Proton ni se reutilizó consentimiento.

## Uso

Desde `aplicacion/apps/lxmi-desktop`, ejecutar `npm run tauri:dev` en una sesión de escritorio con los prerrequisitos existentes.

1. Elegir Wuthering Waves o Zenless Zone Zero en la barra lateral y abrir **Mods** o **Perfiles**.
2. Escribir un nombre y pegar la ruta absoluta de una carpeta de mod o un ZIP. Pulsar **Importar mod**. El juego se asigna explícitamente: LXMI no infiere compatibilidad a partir del INI.
3. Marcar los mods que se quieren incluir, escribir el nombre del perfil y pulsar **Guardar perfil**.
4. Desmarcar o marcar mods y usar **Guardar cambios del perfil** para modificar la selección. Desmarcar conserva el mod en la biblioteca.
5. **Recuperar una selección guardada** carga otro perfil.
6. **Preparar copia del perfil** crea una carpeta `Mods` con los mods guardados. La ruta aparece en pantalla. La preparación no modifica el runtime ni el juego y no provoca recarga.

El nombre de perfil identifica su selección dentro de un juego; guardar el mismo nombre, sin distinguir mayúsculas, actualiza ese perfil. Un perfil vacío es válido. Al cambiar de juego se descarta la selección sin guardar de la vista anterior; los perfiles guardados se conservan. Actualizar biblioteca mantiene la selección del formulario y detecta cambios hechos desde otra ventana.

## Comportamiento implementado

| Función | Resultado |
|---|---|
| Importar carpeta o ZIP | Copia privada con inventario y hashes; originales intactos |
| Reimportar el mismo contenido | Reutiliza el mod existente, conserva su primer nombre |
| Listar biblioteca | Verifica contenido; un mod alterado aparece como incidencia y no se ofrece para seleccionar |
| Guardar perfil | Revalida mods del juego; bloqueo entre procesos y revisión optimista evitan perder cambios |
| Preparar perfil | Copia completa e inmutable; cada mod conserva sus rutas internas dentro de una carpeta `lxmi_<hash>` |
| Repetir preparación idéntica | Revalida y reutiliza la copia; rechaza copias alteradas sin sobrescribirlas |
| Cambiar selección | Produce otra preparación; las anteriores se conservan |

## Almacenamiento y límites

Raíz derivada en backend: `$XDG_DATA_HOME/lxmi`, con el fallback XDG existente. El frontend no puede elegir una raíz de escritura ni un destino de despliegue.

| Ruta relativa a LXMI | Contenido |
|---|---|
| `mods/<game-id>/packages/<hash>/payload/` | Copia importada |
| `mods/<game-id>/packages/<hash>/lxmi-mod.json` | Nombre, juego, versión de esquema e inventario |
| `mods/<game-id>/profiles.json` | Selecciones persistentes y revisión |
| `mods/<game-id>/prepared/<hash>/Mods/` | Copia preparada de una selección |
| `mods/<game-id>/prepared/<hash>/lxmi-profile.json` | Inventario y perfil de esa preparación |
| `mods/<game-id>/staging/` | Copias temporales durante operaciones |
| `mods/<game-id>/library.lock` | Bloqueo entre procesos para mutaciones |

Límites: 256 mods y 64 perfiles por juego; ZIP de 32 MiB, hasta 4 GiB descomprimidos, 128 MiB por archivo, 4096 entradas, 24 niveles y ratio máximo de compresión 1000. Las carpetas importadas y la preparación completa admiten hasta 4 GiB y 4096 entradas. El runtime derivado admite esos 4 GiB más los 512 MiB reservados al runtime base. Metadata limitada a 2 MiB. RAR y 7z requieren extracción previa. Estos límites de mods no cambian los de importación de paquetes oficiales.

Se requiere al menos un INI con contenido o un paquete de reemplazos de shaders con archivos `ShaderFixes/<hash de 16 dígitos hexadecimales>-ps_replace.txt`, `-ps_replace.bin`, `-vs_replace.txt` o `-vs_replace.bin`; estos paquetes pueden incluir README.txt/README.md en la raíz. Al aplicar el perfil, los reemplazos de esa carpeta raíz se copian a `ShaderFixes` del runtime privado. Se rechazan destinos repetidos, también contra los archivos del runtime base y sin distinguir mayúsculas. EXE/DLL/MSI pertenecen al flujo de runtime y se rechazan como mod. Se rechazan symlinks, hardlinks, archivos especiales, escapes, colisiones por mayúsculas y nombres ambiguos/reservados de Windows. Archivos privados 0600 y directorios 0700. Se reutilizan los helpers existentes anclados a descriptores; las promociones de payload no reemplazan contenido existente. La sustitución de `profiles.json` es atómica y ocurre con bloqueo y comprobación de revisión.

SHA-256 detecta cambios respecto al inventario; no autentica al autor. El INI se conserva byte a byte y no se interpreta. No hay detección de conflictos semánticos ni limpieza automática de preparaciones. No se promete recuperación ante pérdida de energía. Un staging abandonado tras caída requiere revisión manual con LXMI cerrado.

## Siguiente trabajo

1. **Completado:** flujo en GTK/WebKit con comandos Tauri reales, mods sintéticos, reinicio y verificación de hashes. Se usan eventos DOM dentro de la ventana propia, sin IPC simulado ni pulsaciones del escritorio. [Evidencia nativa](../06_Verificacion/evidencias/gestor_mods_nativo.json).
2. **Completado:** [aplicación reversible al runtime privado](../04_Arquitectura_y_seguridad/aplicacion_reversible_perfiles.md) y assembler WWMI. En **Compatibilidad**, seleccionar paquetes oficiales, preparar el runtime, elegir perfil guardado, revisar destino/inventario y aplicar. **Revertir selección anterior** alterna la selección conservada; las copias quedan intactas. La selección no inicia el juego. [Verificación](../06_Verificacion/verificacion_activacion_runtime.md).
3. Completar por separado las pruebas del cargador y del contexto Proton necesarias para cada juego.
4. Investigar parser INI con round-trip, grupos y conflictos; después evaluar selección live. [No Reload Mod Manager](https://github.com/Aglglg/No-Reload-Mod-Manager) usa archivos gestionados y modificaciones de INI para comunicarse con el runtime; no se ha portado ese mecanismo a LXMI.

La biblioteca no requiere un juego instalado para sus pruebas sintéticas. Su existencia no acredita compatibilidad de mods, WWMI/ZZMI o Proton.

Ver [verificación](../06_Verificacion/verificacion_gestor_mods_local.md) y [ADR-031](../04_Arquitectura_y_seguridad/ADR/031_local_mod_library_and_profiles.md).

## Repetir la prueba nativa

Con los prerrequisitos Tauri ya instalados, iniciar `npm run dev` desde `aplicacion/apps/lxmi-desktop`. En otra terminal, desde `aplicacion`, ejecutar:

```bash
bash tools/test-mod-manager-native.sh
```

Es una prueba de host opt-in; abre dos ventanas Tauri sucesivas y se cierra sola. Requiere una sesión de escritorio. Usa un directorio temporal nuevo, XDG data/cache aislados y recursos sintéticos. Conserva los JSON y logs bajo la ruta temporal que imprime. El ejemplo comparte el builder y registro de comandos de la aplicación; no sustituye el backend. No ejecuta Proton/Wine, juegos ni experimentos de prefix.

## Primera selección de mods ZZZ desde Downloads (historial)

Se revisaron `mods-zzz/miyabixsuzukav2.rar`, `v78.zip`, `hide_uid_89406.zip` y `censor_remover_v320.zip`. Miyabi contiene 36 archivos y recursos presentes; su INI principal no mostró desbalances en la inspección acotada. `Miyabi Patches.ini` tiene siete `endif` sin apertura y se renombró a `.ini.disabled` **solo en una copia de prueba**. RabbitFX tiene cinco archivos, sin recursos ausentes ni desbalances detectados. Esto no es un parser completo ni acredita compatibilidad con la versión del juego.

Hide UID contiene una comparación vacía y dos `if` sin cierre, por lo que queda fuera del perfil. Censor Remover trae únicamente `ShaderFixes` y su README pide instalarlos en esa carpeta; el gestor actual requiere INI y no se ha implementado ese despliegue. No se simula su instalación colocándolo dentro de `Mods`. Los originales de Downloads se conservan byte a byte. El perfil de prueba se llama **ZZZ — Miyabi y RabbitFX**.

La verificación nativa posterior con estos mods reales recuperó el perfil y los paquetes guardados: [evidencia](../06_Verificacion/evidencias/mods_zzz_nativo.json). La instalación actual aparece en `/media/university/Steam`; compatdata/pfx todavía no están presentes en las rutas Steam inspeccionadas. Es necesario confirmar cómo inicia ZZZ ahora antes de retomar la integración Proton; la reserva histórica estaba vinculada a otro prefix.

## Perfil completo desde Downloads (2026-10-15)

Por solicitud del usuario se importaron y activaron los 12 paquetes de `Downloads/mods-zzz` en **ZZZ — Todos los mods**: 543 archivos en el runtime, con 2 678 652 957 bytes. Las carpetas ya extraídas que duplican ZIP se cuentan una sola vez. Se reutilizan los paquetes existentes de Miyabi y RabbitFX, incluida la desactivación previa de `Miyabi Patches.ini`. El perfil anterior **ZZZ — Miyabi y RabbitFX** se conserva para revertir la selección. [Registro de la importación](../06_Verificacion/evidencias/mods_zzz_perfil_completo_20261015.json).

Hide UID incluye un bloque duplicado sin comparación ni cierres. En la copia importada se conserva únicamente la condición definida `index_count == 1764`, con sus dos `endif`; no se inventa un segundo índice. Los archivos originales de Downloads permanecen intactos. Censor Remover utiliza ahora el despliegue de reemplazos a `ShaderFixes` descrito arriba.

Los 4 GiB permiten incluir Lucia (aproximadamente 655 MiB) y Remielle (aproximadamente 1173 MiB) junto con los demás mods. El perfil de desarrollo compila `sha2` con optimización para acelerar las comprobaciones sobre conjuntos de varios GiB. Las pruebas del gestor y runtime comprueban importación sin INI para shaders, destino real, conservación de la base, copia de arranque y bloqueo de conflictos. La compatibilidad visual de cada nuevo mod requiere observarlo en el juego.
