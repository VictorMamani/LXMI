# Diagnóstico del entorno Linux — LXMI 0.8.8

Fecha: 2026-10-02. Versión de aplicación: **0.8.7**. Trabajo de diagnóstico autorizado después del STOP por `UnexpectedMutation`. No nueva entrada al prefix real; 0.8.9 y 0.9 no iniciados.

## Pregunta y alcance

¿La diferencia entre Proton directo y Proton dentro de Steam Linux Runtime reproduce las mutaciones de fuentes del intento real? Se reutilizaron el helper Windows 0.6, el protocolo 1 y el executor con timeout existentes. No se usó juego, loader, DLL de prueba, PID ni backup del juego para preparar el fixture.

Cada ensayo usa una raíz `/tmp/lxmi-environment-fixture-<nonce>` nueva, privada (0700), del UID actual, sin symlinks. El prefix se crea desde cero en esa raíz. El backend rechaza otros roots y ambos modos de prefix real. No hay comando frontend para este diagnóstico. `game_entry_authorized=false` en cada evidencia.

## Fuente fijada — UPSTREAM VERIFIED

Steam Runtime Tools, tag `v0.20260805.0`, tag object `b5d80784f09ea6484fc81145893fc5c20985231a`, commit `02cb49d93300fcedd01ce896d770e19b74160af7`, commit date `2026-08-05T14:11:36+01:00`. Source descargado a `/tmp/lxmi-steam-runtime-tools-review` para inspección; no build/run del source descargado.

- [pressure-vessel/wrap.c](https://gitlab.steamos.cloud/steamrt/steam-runtime-tools/-/blob/02cb49d93300fcedd01ce896d770e19b74160af7/pressure-vessel/wrap.c#L293): reemplaza stdin por `/dev/null` salvo terminal TTY o modo devel.
- [pressure-vessel/flatpak-run.c](https://gitlab.steamos.cloud/steamrt/steam-runtime-tools/-/blob/02cb49d93300fcedd01ce896d770e19b74160af7/pressure-vessel/flatpak-run.c#L1059): importa las fuentes de `/usr/share/fonts` a `/run/host/fonts` y genera la configuración fontconfig correspondiente. Este código también se usa por pressure-vessel; el nombre del archivo no prueba que Steam sea Flatpak.
- [Documentación Valve](https://github.com/ValveSoftware/steam-runtime/blob/master/doc/reporting-steamlinuxruntime-bugs.md): describe la capa pressure-vessel + runtime. Referencia contextual consultada 2026-10-02; las conclusiones de implementación se basan en source fijado y scripts locales, no en main flotante.

## Selección explícita y pin local — LOCAL OBSERVED

| Componente | Evidencia |
|---|---|
| Proton | Experimental, `1790244902 experimental-11.0-20260924-x86_64`; comparación exacta de `version` antes del ensayo, sin sustitución |
| Proton toolmanifest | `require_tool_appid=4183110` (Steam Linux Runtime 4), leído sin modificar |
| Container instalado | `SteamLinuxRuntime_4`, depot/runtime `4.0.20260805.254769`, pressure-vessel/scripts `0.20260805.0` |
| `_v2-entry-point` SHA-256 | `caa39b5cde8ea955288b574b49b3416fd16e7be3f53a17249d673f5ecfdce2c1` |
| `VERSIONS.txt` SHA-256 | `ef61e9bce47d60d02707661d116c8048ffa39291b84acee687a1ed4e430af6be` |
| Helper sin modificaciones | SHA-256 `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2` |

Los scripts locales `_v2-entry-point` y `run` fueron inspeccionados. `run` admite `PRESSURE_VESSEL_VARIABLE_DIR`: se fijó a `<fixture>/runtime-variable`, evitando su default dentro de la instalación del runtime. El entorno parte del allowlist del bridge; no se copia el entorno del juego ni se activa ningún AppID de juego.

Esto identifica la capa usada **por el ensayo**, no reconstruye inequívocamente todas las capas de la ejecución histórica de ZZZ.

## Transporte del protocolo

Primer fixture `04269c47b2ce142aef3175c37c3094d6`: helper ejecutado en el contenedor, resultado `NonceMismatch`; no pasó al segundo paso. La explicación del stdin descartado está establecida en el source fijado. Los paths reales monitoreados permanecieron iguales. No se reintentó en ese prefix.

Una preparación posterior encontró el límite de tamaño del binario Linux de debug (71,670,096 bytes frente al límite inicial 64 MiB), antes de lanzar Proton. Se ajustó exclusivamente el límite del transporte Linux a 256 MiB; el límite/identidad del helper Windows permanece intacto.

Corrección del ensayo: una copia identificada del mismo ejemplo Linux recibe el archivo privado `bridge-request.json`, verifica su hash/protocolo/nonce/marker fijo, reabre ese archivo como stdin y reemplaza su proceso por el argv fijo de Proton + helper. No shell, argumento de comando, PID ni destino arbitrario. La solicitud conserva el protocolo Windows existente y no añade poder al helper.

Se usa el mismo executor para el comando externo, con timeout máximo 60 s. El bridge directo mantiene 20 s. Se corrigió el mensaje de timeout para usar el límite real de la invocación. No se utiliza `--devel` ni un modo de terminal más permisivo para preservar stdin.

## HOST TEST — resultados

Fixture completo: `b8b55aadbda6ecadb0c2bf483824d3a4`. Se creó antes de ejecutar el primer helper. Misma identidad de prefix durante los cinco pasos; solicitudes con nonce fresco en cada ejecución, marker propio del fixture.

| Paso | Helper / nonce / marker hash / exit | Fuentes, por cada una de las 3 tablas | Otros cambios semánticos |
|---|---|---|---|
| Contenedor inicial | PASS / 0 | Inicialización; sin comparación anterior | Inicialización; sin comparación anterior |
| Repetir contenedor | PASS / 0 | 0 cambios | 4 ContainerId; clave NetworkCards vacía removida; 2 secciones aún no clasificadas |
| Cambiar a directo | PASS / 0 | +135 entradas, 368 rutas modificadas, 0 removidas | 4 ContainerId |
| Repetir directo | PASS / 0 | 0 cambios | 4 ContainerId |
| Volver a contenedor | PASS / 0 | 0 nuevas, 368 rutas modificadas, -135 entradas | 4 ContainerId |

**COMPROBADO:** cambio a directo reproduce exactamente los conteos y la dirección `/run/host` a `/usr/share/fonts` del intento real. El mismo filename permanece en las 368 modificaciones de cada tabla. El retorno al contenedor invierte las 368 rutas y remueve las 135 entradas por tabla. No se suman las tablas como si describieran archivos de fuentes independientes.

**HOST TEST:** los cuatro ContainerId también cambian al repetir el mismo entorno. La remoción de NetworkCards apareció al repetir contenedor, antes del cambio a directo. Por tanto, esos cambios no pueden atribuirse únicamente a la transición de rutas de fuentes.

Se esperaron 5 s tras cada helper y se exigieron dos lecturas de registro idénticas separadas por 2 s, con intentos acotados. No se enviaron señales ni comandos a wineserver. La estabilidad de hashes no prueba por sí sola que el wineserver haya terminado: esa relación permanece **UNKNOWN** en este ensayo.

## Evidencia durable y cobertura

Reporte sin valores de registro, GUIDs, entorno completo ni process lists: [fixture_environment_0_8_8.json](../06_Verificacion/evidencias/fixture_environment_0_8_8.json).

SHA-256: `688d410e401b39d512907d0e3eecd565c0144cffc158f027dc3874c91e64fb2c`. Incluye argv externo/bridge fijo, versiones/hashes, identidad del prefix privado, hash de cada hive y diagnósticos por paso. No contiene raw registry dumps.

**READ-ONLY SOURCE:** comprobación antes/después y entre pasos de config.vdf, dos localconfig.vdf, appmanifest_4162040, metadata del ejecutable del juego, los tres hives del prefix real y su identidad. Sin diferencias en esa cobertura. Se usó O_NOFOLLOW/O_NOATIME para contenido regular acotado. El executable solo fue monitoreado por metadata, no rehashado: no se afirma cobertura completa de instalación, paquetes, Steam ni filesystem.

Los caches de runtime y solicitudes quedaron bajo `/tmp` privado. Las dos raíces que llegaron a ejecutar helpers ocupaban aproximadamente 1.24 GiB y 713 MiB (du por raíz; puede haber hardlinks compartidos, no equivale a espacio incremental sumado). Se conservan como evidencia temporal, sin limpieza global. No se modificó el backup recuperable. No se afirma inmutabilidad de metadata de locks internos del tool Proton instalado.

## Corrección del guard — COMPROBADO

La ruta real disponible aún ejecuta Proton directo y no tiene política calificada para las mutaciones observadas. `LinuxEnvironmentUnqualified` bloquea validación y ejecución en el dominio. Los comandos Tauri Review/Run devuelven `linux_runtime_environment_unqualified` antes de nuevos snapshots/staging/ejecución. El frontend existente muestra el código y razón; no se rediseñó la UI.

Un nuevo consentimiento, baseline, backup o restart no elimina este requisito. La comparación fixture no reclasifica el recibo original `UnexpectedMutation`. El guard de Hook y la selección de runtime/loader permanecen intactos.

## Qué queda antes de otra entrada real

- **COMPROBADO:** reproducida y acotada la causa de las diferencias de fuentes; usar el contenedor evita esa transición en las repeticiones fixture.
- **NEEDS EVIDENCE:** origen preciso y consecuencias de ContainerId, remoción NetworkCards y las 2 secciones no clasificadas de la repetición inicial.
- **NEEDS EVIDENCE:** política estrecha de mutaciones respaldada por esas pruebas y qualification del entorno que se propondría para una nueva entrada real.
- **UNKNOWN:** equivalencia completa con el entorno histórico del juego, finalización de wineserver y riesgo de entrada con juego activo.
- Cualquier nueva entrada real requiere un plan calificado, fresh preflight y consentimiento independiente en Tauri. No se ejecuta en esta investigación.

## Validación

COMPROBADO: fmt/check/clippy workspace all-targets/all-features `-D warnings`; 304 tests PASS, 11 ignored; frontend typecheck/lint/format:check/build; git diff --check. Los tests normales solo usan fixtures sintéticos. El experimento host fue un ejemplo opt-in explícito, no un test normal ni un loader experiment.

Native Tauri: `npm run tauri:dev`, Scan Steam, panel ZZZ Prefix Preflight y Review. AT-SPI confirmó el texto `linux_runtime_environment_unqualified` con su razón. Sin aceptar consentimiento ni pulsar Run. Tauri/Vite cerrados tras validar. La primera extracción falló por usar el método Text incorrecto de GI; la extracción final con Atspi.Text confirmó el mensaje, no un fallo del render.

Versión0.8.7; 0.8.8 BLOCKED; 0.8.9/0.9 no iniciados. Cambios previos conservados. Sin commit ni push.
