# LXMI 0.6 — Bridge Linux ↔ Windows

## Alcance y estado

Esta investigación define un ensayo local de conectividad entre LXMI nativo y un helper Windows inocuo iniciado mediante un Proton elegido explícitamente. No prueba compatibilidad de ZZZ, carga de DLL, injector, launcher upstream ni inyección.

Fecha de consulta upstream: **2026-09-28**.

## Fuentes fijadas

| Fuente | Pin | Hallazgo utilizado |
|---|---|---|
| ValveSoftware/Proton README | tag proton-11.0-2, commit corto de la release db9e6ff | Proton se presenta como herramienta para Steam y Valve recomienda normalmente usar la versión proporcionada por el cliente; la documentación describe builds locales y sus archivos estructurales. |
| ValveSoftware/Proton proton | tag proton-11.0-2, archivo proton | El script tiene una operación run, consume STEAM_COMPAT_DATA_PATH y STEAM_COMPAT_CLIENT_INSTALL_PATH, mantiene CompatData/pfx y contiene acciones de mantenimiento sobre la distribución. |
| Proton release | tag proton-11.0-2 | La página de release identifica db9e6ff; el pin permite volver a inspeccionar el código consultado. |

Enlaces:

- [README.md en proton-11.0-2](https://github.com/ValveSoftware/Proton/blob/proton-11.0-2/README.md)
- [script proton en proton-11.0-2](https://github.com/ValveSoftware/Proton/blob/proton-11.0-2/proton)
- [release proton-11.0-2](https://github.com/ValveSoftware/Proton/releases/tag/proton-11.0-2)

La revisión del código de Proton es evidencia de implementación para ese tag, **no una garantía de API estable para ejecutar juegos fuera de Steam**. El bridge implementa únicamente un test controlado de un helper de diagnóstico. No se extrapola a la topología de ZZZ ni a un launcher XXMI.

## Forma de invocación evaluada

La inspección read-only del Proton Experimental local (`experimental-11.0-20260924-x86_64`) mostró una diferencia crítica entre dos verbos del script:

- `run` inicia `steam.exe` dentro del prefix y añade los argumentos de CLI a ese proceso; no ejecuta un helper arbitrario como proceso objetivo.
- `runinprefix` ejecuta directamente `wine` con los argumentos restantes. Es la ruta usada para este helper autónomo.

La implementación sigue esta invocación en argv, sin shell:

~~~text
<Steam library>/steamapps/common/<Proton tool>/proton runinprefix <LXMI managed helper.exe>
~~~

La selección se limita al test explícito de LXMI; no representa una selección de Steam para ZZZ. El comportamiento de `runinprefix` se verificó en el código de la versión local, pero es un subcomando interno del script y no se asume como API estable para todas las versiones de Proton.

El entorno del hijo se limpia y se crea una allowlist mínima. Incluye PATH, LANG, HOME/XDG acotados al test, el marker no secreto del protocolo, WINEDEBUG=-all y:

| Variable | Valor del test |
|---|---|
| STEAM_COMPAT_CLIENT_INSTALL_PATH | Steam root asociado al Proton elegido |
| STEAM_COMPAT_DATA_PATH | $XDG_DATA_HOME/lxmi/test-prefixes/bridge-v1/compatdata |

LXMI no define WINEPREFIX ni una variable PROTON_*. El script de Proton deriva su contexto de CompatData y administra allí pfx/. LXMI crea solo el directorio vacío de CompatData, junto con sus carpetas XDG aisladas; **no fabrica una estructura de prefix Wine**.

El helper no abre procesos del juego. Lee el request JSON por stdin, valida versión/nonce/ruta relativa, deriva el root desde su ubicación administrada, consulta un único runtime-manifest.json, calcula hashes y devuelve un JSON por stdout. El nonce correlaciona una respuesta con una invocación; no autentica el proceso. El hash local del helper detecta divergencias respecto del manifest local de build, pero no es una firma del publisher.

## Efectos de ejecutar Proton

El script upstream fijado incluye cleanup_legacy_dist, que puede retirar un dist/ antiguo, y do_steampipe_fixups, que puede escribir cambios si detecta metadata desactualizada. También crea o actualiza pfx/ dentro de CompatData. Por eso:

- el modo ejecutable de LXMI solo admite el test-prefix bajo storage LXMI;
- la UI exige una aceptación explícita de esos efectos antes de ejecutar;
- nunca se pasa compatdata/4162040 ni una ruta de prefix de juego;
- el host test 0.6 se ejecutó tras compilar y stagear el helper; Proton inicializó solo el contexto aislado LXMI y mantuvo su propia distribución;
- no se afirma que ejecutar Proton no alteraría la instalación de Proton.

En lectura local de Proton Experimental, el timestamp de files/steampipe_fixups_mtime coincidía con el de steampipe_fixups.json y no se encontró dist/. Esas observaciones no sustituyen un test, ni demuestran que una futura versión o una llamada real no escriba.

## Resolución de rutas

LXMI entrega al helper una ruta relativa bajo su propio root, no una ruta que el frontend pueda escoger libremente. El helper reconstruye el path usando su ubicación dentro de helpers/, valida confinamiento y symlinks, y devuelve la ruta que ve Windows/Wine. No se asume Z:. La prueba compara el SHA-256 de runtime-manifest.json leído por Linux con el calculado por el helper.

d3dx.ini, d3d11.dll, 3dmloader.dll, ZenlessZoneZero.exe, launch options, registro Wine y Steam no forman parte del request ni de la operación helper.

## Distinción de runtimes

La UI permite escoger explícitamente una herramienta Proton válida para **Bridge test runtime**. No escribe esa selección en GameRuntimePlan, no la presenta como Proton elegido por Steam para ZZZ y no usa el candidato único como inferencia.

ExistingGameCompatdataReadOnlyContext permanece como modelo diagnóstico, pero el executor lo rechaza. No existe modo que acepte un path de prefix arbitrario desde IPC.

## Estado comprobable

- **UPSTREAM VERIFIED:** tag de Proton, forma de invocación que implementa su script, variables de CompatData y puntos de mantenimiento descritos arriba.
- **COMPROBADO:** Proton Experimental `experimental-11.0-20260924-x86_64` inició el helper staged por `runinprefix`; el handshake nonce v1 pasó, la ruta Windows fue leída por el helper y el hash del marker coincidió. Un path inexistente generó respuesta `runtime_not_visible` y exit 2.
- **COMPROBADO:** la ventana Tauri nativa mostró el scan y el resultado. El helper ausente y el runtime administrado ausente se observaron como estados distintos.
- **FIXTURE:** runtime-manifest, hashes, nonce y respuestas de executor falso.
- **NO COMPROBADO:** compatibilidad de ZZMI/ZZZ con Steam/Linux/Proton, carga de DLL, selección de runtime del juego o interacción con proceso del juego.
