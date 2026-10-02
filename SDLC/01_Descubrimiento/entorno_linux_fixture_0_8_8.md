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

## LXMI 0.8.8.1 — calificación con estados iniciales equivalentes

Fecha: **2026-10-02**. La aplicación continúa en **0.8.7** y 0.8.8 permanece **BLOCKED / PARTIAL**. Esta sección complementa la historia anterior: el ensayo original cambiaba sucesivamente un mismo prefix y dejaba dos secciones sin identificar. La matriz nueva usa tres clones independientes de un seed privado, sin usar el prefix, registros, backup ni entorno de ZZZ. No se repite la entrada real detenida por `UnexpectedMutation`.

### Fuentes fijadas — UPSTREAM VERIFIED

| Componente | Pin consultado | Uso de la evidencia |
|---|---|---|
| Steam Runtime Tools | `v0.20260805.0`, commit `02cb49d93300fcedd01ce896d770e19b74160af7` | Container, interfaz de compatibility tools, stdin, font mounts y wrapper soportado |
| Proton | rama `experimental_11.0`, commit `22f5f694d698d786a9e8ff91d559147c8189a889` | `runinprefix`, prefix, mantenimiento de distribución y locks |
| Wine, submodule del Proton fijado | commit `adea6cbac9d00bb22910145354d00c463fcf3afc` | Fonts, ContainerId, NDIS, serialización y wineboot |

**LOCAL OBSERVED:** el script `proton` instalado es byte a byte igual al del commit fijado, SHA-256 `077db31dda75c570c1a91014f9a814b785307bc91e5dfb1480b984dc34166a54`. La versión local es `1790244902 experimental-11.0-20260924-x86_64`. Esto fija el script y metadata local; no demuestra una reconstrucción reproducible de todos los binarios upstream.

**UPSTREAM VERIFIED:** Proton 11 declara Steam Linux Runtime 4, AppID de herramienta `4183110`, basado en steamrt4. Steam compone compatibility tools: el runtime externo construye el container y ejecuta el tool interior. `STEAM_COMPAT_DATA_PATH` identifica el contexto de datos que Proton convierte en prefix `pfx`; no es selección de Proton ni autorización de un juego. Dentro del container, `LD_LIBRARY_PATH` combina runtime y bibliotecas del proveedor. [Container runtime](https://gitlab.steamos.cloud/steamrt/steam-runtime-tools/-/blob/02cb49d93300fcedd01ce896d770e19b74160af7/docs/container-runtime.md), [interfaz de Steam](https://gitlab.steamos.cloud/steamrt/steam-runtime-tools/-/blob/02cb49d93300fcedd01ce896d770e19b74160af7/docs/steam-compat-tool-interface.md).

**UPSTREAM VERIFIED:** la documentación ofrece el wrapper `SteamLinuxRuntime/run -- Proton/proton ...` y una alternativa de launch client hacia un servicio configurado explícitamente. No se configura ni usa ese servicio en LXMI. `_v2-entry-point` advierte que su interfaz es provisional; se prueba solo para el build fijado. La ruta `run` documentada es la candidata preferida para una futura integración. [Guía de desarrolladores](https://gitlab.steamos.cloud/steamrt/steam-runtime-tools/-/blob/02cb49d93300fcedd01ce896d770e19b74160af7/docs/slr-for-game-developers.md).

### Copia privada de Proton y límites de escritura

**UPSTREAM VERIFIED:** antes de `runinprefix`, Proton llama mantenimiento de su distribución. `do_steampipe_fixups` toma `dist.lock`; `UnixFileLock` abre con `O_RDWR | O_CREAT | O_TRUNC`. Ejecutar el script instalado no garantiza que solo se escriba el prefix privado. También son posibles caches de Python. [Proton fijado](https://github.com/ValveSoftware/Proton/blob/22f5f694d698d786a9e8ff91d559147c8189a889/proton), [filelock.py](https://github.com/ValveSoftware/Proton/blob/22f5f694d698d786a9e8ff91d559147c8189a889/filelock.py).

**COMPROBADO:** el harness actual exige una copia completa y verificada de Proton en `<seed>/proton-tool`. Reutiliza la copia mediante descriptores de recuperación existente: no sigue symlinks, preserva metadata soportada, verifica el árbol y detecta cambio de fuente durante la copia. Rechaza ejecutar el tool instalado sin recibo de mirror privado. `PYTHONDONTWRITEBYTECODE=1` evita caches Python. Los locks de Proton quedan en esa copia, no en la distribución instalada.

Las matrices preliminares que ejecutaron el tool instalado se conservan en `environment_qualification_0_8_8_1_installed_tool.json` y su revisión de fuentes. **No califican esta frontera de escritura.** No se restauró `dist.lock` ni se oculta ese límite histórico.

**HOST TEST:** copia privada con manifest SHA-256 `9d0bb73e241bac979afeda526b589fce6f26ee2d5e8822b7aadc710b179421bb`; fuente sin cambios durante la copia completa. Metadata vigilada de `proton`, `version`, `dist.lock`, `files/steampipe_fixups_mtime`, `steampipe_fixups.json` y `__pycache__` permaneció igual antes/después de las seis ejecuciones finales. Esta comprobación posterior es selectiva, no un hash de toda la distribución instalada durante todas las ejecuciones.

### Receta reproducible y argv

Cada root es `/tmp/lxmi-environment-fixture-<32 hex>`, canonical, sin symlink, UID actual y permisos 0700. `compatdata` está en `<fixture>/test-prefixes/bridge-v1/compatdata`, y `pfx` debajo. El tool privado es compartido explícitamente por las variantes; sus prefixes son independientes. El helper Windows conserva SHA-256 `2a1939728e18bd3190b8dbb102d943bd5d65fa509617dbe42d48c45d02463fd2` y protocolo 1.

| Variante | Executable y argumentos directos | Request |
|---|---|---|
| DirectProton | `<private-tool>/proton`, `runinprefix`, `<fixture>/helpers/lxmi-bridge-helper.exe` | Stdin |
| SteamLinuxRuntimeContainer | `<SteamLinuxRuntime_4>/_v2-entry-point`, `--verb=run`, `--`, `<fixture>/fixture-container-transport` | PrivateRequestFile |
| Container con wrapper documentado | `<SteamLinuxRuntime_4>/run`, `--`, `<fixture>/fixture-container-transport` | PrivateRequestFile |

El transport ejecuta exactamente el mismo argv interior de Proton/helper. No hay shell, comandos derivados del request ni procesos elegidos por usuario. `STEAM_COMPAT_CLIENT_INSTALL_PATH` es metadata requerida por Proton; no inicia Steam. No se establece `SteamAppId=4162040`.

Allowlist exterior completa: `PATH=/usr/bin:/bin`, `LANG=C.UTF-8`, `PYTHONDONTWRITEBYTECODE=1`, `WINEDEBUG=-all`, `LXMI_BRIDGE_TEST_MARKER=lxmi-bridge-v1`, `HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_DATA_HOME` (todos privados), `STEAM_COMPAT_CLIENT_INSTALL_PATH` y `STEAM_COMPAT_DATA_PATH` (privado). `env_clear` elimina herencia. El container añade únicamente `PRESSURE_VESSEL_VARIABLE_DIR=<fixture>/runtime-variable`, hash de request y nonce de request. No se copia `/proc/<Steam>/environ` ni el entorno histórico de ZZZ.

**LOCAL OBSERVED:** Steam Linux Runtime 4 `4.0.20260805.254769`, pressure-vessel/scripts `0.20260805.0`. SHA-256 de `run`: `d5069c037c53f204a9d7d0ecb501b430e57dc2d702d49c9cccd57d569ca87eec`; `VERSIONS.txt`: `ef61e9bce47d60d02707661d116c8048ffa39291b84acee687a1ed4e430af6be`; pressure-vessel-wrap: `0439811997a081e4f90f898b7e98e34e46cff3745a0611c17309947fd47f4da9`. Los pins se comprueban durante el ensayo, sin sustituir un runtime distinto.

Dentro del transport propio se registran solo variables documentadas y cuatro mount points. `/usr`, `/run/host/usr`, `/run/host/fonts` y `/run/host/local-fonts` aparecen `ro,nosuid,nodev,relatime`, filesystem ext4. `container=pressure-vessel`; las rutas generadas de `LD_LIBRARY_PATH` apuntan a aliases de overrides x86_64/i386. No se vuelca el mount table completo. No hay `FONTCONFIG_*` explícito en esa allowlist observada; la importación/remap de fuentes está en el código de pressure-vessel.

### Estados iniciales y matriz — HOST TEST

Una inicialización container privada prepara el seed. Se copian **tres árboles completos**, verificados con el mismo manifest `9ad5ac3573494d72f5ce9997a336ba4c1206eded9961731fdd110089e2341eaa`. Cada estrategia ejecuta run 1 y run 2 sobre su propio clone. Run 2 mide repetición; ninguna estrategia hereda las mutaciones de otra. Todos leen los mismos bytes de marker, SHA-256 `4e51c3be0fdc4346507d87367f6fba7375c184cbd0efad91503a5d5fd27bb99e`, con request nonce fresco en cada ejecución.

| Evidencia | Direct run 1 / 2 | `_v2` run 1 / 2 | `run` run 1 / 2 |
|---|---|---|---|
| Helper / nonce / marker/hash / exit 0 | PASS / PASS | PASS / PASS | PASS / PASS |
| Fonts, cada tabla | +135, 368 rewrites / 0 | 0 / 0 | 0 / 0 |
| NetworkCards vacío en hive | omitido / sin cambio | omitido / sin cambio | omitido / sin cambio |
| Wow6432Node RunServices | creado vacío / sin cambio | creado vacío / sin cambio | creado vacío / sin cambio |
| Wow6432Node RunServicesOnce | creado vacío / sin cambio | creado vacío / sin cambio | creado vacío / sin cambio |
| Device ContainerId | 4 cambios / 4 cambios | 4 / 4 | 4 / 4 |
| dosdevices mappings | iguales / iguales | iguales / iguales | iguales / iguales |
| Identidad root de cada prefix | igual / igual | igual / igual | igual / igual |
| Secciones semánticas UNKNOWN | 0 / 0 | 0 / 0 | 0 / 0 |
| Calificación de receta privada | Unqualified | SteamRuntimeQualified | SteamRuntimeQualified |

Seis ejecuciones medidas más la inicialización del seed: todas PASS. Artefacto completo redactado: [environment_qualification_0_8_8_1.json](../06_Verificacion/evidencias/environment_qualification_0_8_8_1.json), SHA-256 `0797a5ed44ea490ad53b5ac582d80aa77360f57cfe3d25a9ac2e81c15e509403`. Root seed: `/tmp/lxmi-environment-fixture-916b0461d62b57b48d042649c8f72b75`. El artefacto conserva argv, env allowlist, prefixes/identidad, hashes de hives, cambios por sección/campo y metadata selectiva; no guarda contenido completo de registros, environment ni procesos.

### Fuentes — ENVIRONMENT SPECIFIC

Las tablas exactas son `Software\Microsoft\Windows NT\CurrentVersion\Fonts`, `Software\Microsoft\Windows\CurrentVersion\Fonts` y `Software\Wine\Fonts\External Fonts` en sus hives correspondientes. Los conteos son **por tabla**, no 503 archivos ni tres veces esa cantidad de fuentes físicas. El directo agrega 135 valores por tabla, incluyendo 111 que apuntan a fuentes TeX bajo `/usr/share/texmf/fonts`, y reescribe 368 rutas existentes. El filename permanece igual al cambiar la exposición `/run/host/...` por `/usr/share/fonts/...`. La repetición directa queda estable; no vuelve equivalente al container el estado directo.

**UPSTREAM VERIFIED:** `win32u/font.c` reconstruye entradas de Fonts/External Fonts a partir de las fuentes visibles; pressure-vessel importa fuentes host y aplica remap de fontconfig. **DERIVED:** la combinación de esos mecanismos y la matriz controlada explica las diferencias de entorno observadas. No se identifica cada llamada productora mediante trace/attach, ni se autoriza aplicar estas diferencias al juego. [Wine font.c](https://github.com/ValveSoftware/wine/blob/adea6cbac9d00bb22910145354d00c463fcf3afc/dlls/win32u/font.c), [font mounts](https://gitlab.steamos.cloud/steamrt/steam-runtime-tools/-/blob/02cb49d93300fcedd01ce896d770e19b74160af7/pressure-vessel/flatpak-run.c#L1059).

El clasificador acepta solo valores string con extensiones/rutas de fuentes concretas. Incluye TeX y separadores Windows duplicados serializados por Wine. Un valor DWORD, ruta ajena o campo nuevo permanece UNKNOWN; no se ignora toda la rama Fonts.

### ContainerId — VOLATILE

No son cuatro variables de Linux: son valores registry `ContainerId` en estos cuatro dispositivos:

- `System\ControlSet001\Enum\HID\VID_845E&PID_0001\0&0000&0&0&0`
- `System\ControlSet001\Enum\HID\VID_845E&PID_0002\0&0000FFFFFFFF0002845E&0&0&0`
- `System\ControlSet001\Enum\WINEBUS\VID_845E&PID_0001\0&0000&0&0&0`
- `System\ControlSet001\Enum\WINEBUS\VID_845E&PID_0002\0&0000FFFFFFFF0002845E&0&0&0`

**UPSTREAM VERIFIED:** `make_unique_container_id` incorpora datos del dispositivo y `QueryPerformanceCounter`; HID/PnP propaga ese ID y lo publica como `SPDRP_BASE_CONTAINERID`. Un ID no nulo que ya es único puede conservarse. **HOST TEST:** los cuatro GUID cambian en ambas ejecuciones finales de cada estrategia. La matriz preliminar incluyó una repetición sin cambios; por ello no se afirma que toda invocación container necesariamente regenere IDs. [winebus](https://github.com/ValveSoftware/wine/blob/adea6cbac9d00bb22910145354d00c463fcf3afc/dlls/winebus.sys/main.c), [PnP](https://github.com/ValveSoftware/wine/blob/adea6cbac9d00bb22910145354d00c463fcf3afc/dlls/ntoskrnl.exe/pnp.c).

Clasificación `VolatileExpected` limitada a GUID válidos, mismo dispositivo y cambio exclusivo de `ContainerId`. No demuestra inocuidad funcional universal, sesión Linux nueva ni identidad/lifetime del wineserver. Otros campos de esos dispositivos siguen UNKNOWN.

### NetworkCards y las dos secciones identificadas

**HOST TEST:** la sección vacía `Software\Microsoft\Windows NT\CurrentVersion\NetworkCards` deja de aparecer en el hive guardado durante run 1 de **todas** las estrategias; no es exclusiva de DirectProton. Run 2 no la vuelve a introducir.

**UPSTREAM VERIFIED:** NDIS crea subclaves `REG_OPTION_VOLATILE`. El serializer Wine omite un parent vacío que tiene subclaves, y luego omite las subclaves volátiles. **DERIVED:** esto explica que no se serialice esa sección vacía; no prueba que desaparezca la clave del registro vivo ni un fallo de red. Regla `StableExpected` únicamente para ese parent exacto, previamente vacío. Remover contenido no vacío queda UNKNOWN. [NDIS](https://github.com/ValveSoftware/wine/blob/adea6cbac9d00bb22910145354d00c463fcf3afc/dlls/ndis.sys/main.c), [serializer](https://github.com/ValveSoftware/wine/blob/adea6cbac9d00bb22910145354d00c463fcf3afc/server/registry.c).

Las dos secciones antes sin nombre son **creaciones vacías**, no eliminaciones:

- `Software\Wow6432Node\Microsoft\Windows\CurrentVersion\RunServices`
- `Software\Wow6432Node\Microsoft\Windows\CurrentVersion\RunServicesOnce`

**UPSTREAM VERIFIED:** wineboot recorre las vistas nativa/32-bit con `ProcessRunKeys` y `process_run_key` usa `RegCreateKeyExW`. **HOST TEST:** creación vacía en run 1 y estabilidad en run 2. Regla `StableExpected` exacta absent-to-empty; no acepta contenido ni comandos en estas claves. [wineboot](https://github.com/ValveSoftware/wine/blob/adea6cbac9d00bb22910145354d00c463fcf3afc/programs/wineboot/wineboot.c).

El contador agregado histórico `unknown_sections=2` se conserva en `diagnosis`; el nuevo detalle `sections` identifica y clasifica esas claves. La calificación usa este detalle estricto, no un contador agregado ni wildcard.

### Metadata, transporte y calificación

**HOST TEST:** root identity y mappings no cambian. `.`, `dosdevices`, `system.reg`, `user.reg` y `userdef.reg` pueden cambiar timestamps o inode de archivo por reescritura; `userdef.reg` conserva contenido. Son `VolatileExpected` dentro de esta política selectiva; device/mode y root inode deben permanecer iguales. Cualquier nueva metadata/campo o cambio en `drive_c`, `.update-timestamp`, `creation_sync_guard` queda UNKNOWN/conflicting. Los tres hives se analizan semánticamente completos, dentro del límite 8 MiB. No se declara ausencia de cambios en todo el árbol posterior ni prueba de retorno del wineserver a inactivo.

**COMPROBADO:** transporte `PrivateRequestFile` mantiene el helper existente sin modificaciones. El shim Linux fijo se copia y verifica por hash; el request no contiene executable/argv. Archivo `bridge-request-<nonce>.json` en root LXMI privado: create-new, 0600, máximo 64 KiB, schema/protocol, nonce fresco, hash, UID, nlink=1, edad máxima 120 s y rechazo de symlink. Se valida y rebobina el **mismo descriptor**, se consume mediante unlink antes de exec y se conecta como stdin. No se reabre por nombre ni hay fallback silencioso. Cleanup también en error/timeout; fallos de cleanup son explícitos. Un recibo de mounts/env propio se correlaciona por nonce y se elimina al recogerlo. `pressure-vessel` deja stdin `/dev/null` como está diseñado; no se activa devel/TTY. El executor/timeout existentes se reutilizan.

**QUALIFIED (fixture scope):** ambos wrappers container alcanzan `SteamRuntimeQualified` porque dos runs controlados pasan, pins coinciden, allowlist/mounts están justificados y no hay diferencias semánticas desconocidas en la cobertura seleccionada. DirectProton permanece `Unqualified` por Fonts `EnvironmentSpecific` frente al seed container. Cambios inesperados, runtime drift, clones distintos, respuesta incorrecta o un campo UNKNOWN impiden calificación.

**BLOCKED (real prefix):** la implementación real aún es DirectProton y mantiene `linux_runtime_environment_unqualified`. La receta privada container no es un permiso ni un executor para `compatdata/4162040`. Review/Run reales rechazan en backend; un baseline/checkbox no lo sobrescribe. UI muestra calificación histórica y fuente/hash, código exacto, límites y Run deshabilitado; no revalida el host al consultar esta evidencia compilada.

**UNKNOWN / NO COMPROBADO:** equivalencia completa con el entorno histórico de Steam/ZZZ, entrada real con container calificado, helper junto a juego activo, compatibilidad ZZMI/ZZZ y loader strategy. Una futura continuación de 0.8.8 necesita integrar la receta calificada de forma revisable, checks inmediatos y consentimiento nativo nuevo para una sola prueba; este subincremento no ejecuta esa continuación. Versión **0.8.7**; 0.8.9/0.9 no iniciados.

### Validación final 0.8.8.1

**COMPROBADO:** `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`: **325 PASS / 0 failed / 12 ignored**. Tests normales offline no ejecutan Proton ni inspeccionan Steam/procfs real. Frontend `npm run typecheck`, `lint`, `format:check`, `build` y `git diff --check`: PASS. El helper Windows no fue modificado ni reconstruido en este subincremento.

**COMPROBADO — Tauri nativo:** `npm run tauri:dev` abrió LXMI 0.8.7. Validación read-only mediante AT-SPI: Scan Steam, Advanced Prefix Preflight y Review; se verificaron dos recetas privadas `steam_runtime_qualified`, SHA del artefacto final, executor actual `unqualified`, el motivo concreto de falta de integración real-container y botón Run deshabilitado. No consentimiento ni Run, no browser-only. Tauri/Vite propios cerrados después; no control de Steam/Wine/juego. Recibo mínimo en `environment_qualification_0_8_8_1_native.json`; no se guarda el texto completo de otras aplicaciones.

**BLOCKED:** 0.8.8 sigue parcial, versión 0.8.7. La mutación real original sigue preservada, sin retry ni restore. El subincremento califica recetas privadas y mantiene separado el executor real todavía no calificado. Sin commit/push ni inicio de 0.8.9/0.9.

## LXMI 0.8.8.2 — integración del executor calificado (2026-10-02)

Esta sección sustituye el estado del executor descrito al cerrar 0.8.8.1; conserva la investigación y el intento real previo. **0.8.8 sigue PARTIAL / BLOCKED, versión 0.8.7.** No se inició 0.8.9/0.9.

### Una receta y una frontera de autorización

**COMPROBADO:** `QualifiedLinuxRuntimeRecipe` comparte `RuntimeFixtureExecutor`, el wrapper documentado `run`, el transporte privado y el bridge 0.6 entre self-test y futura entry real. `run` llama `_v2-entry-point` internamente; no se ejecutan ambos wrappers consecutivamente. El transporte Linux es un ELF mínimo del crate bridge, sin dependencias GUI; el helper Windows permanece idéntico. Los comandos usan argv fijo, sin shell ni argumentos/path/PID aceptados desde frontend.

La identidad incluye schema, SHA-256 de VERSIONS del runtime, pressure-vessel-wrap, `_v2-entry-point`, `run`, build Proton, manifest completo de copia Proton, versiones de política environment/transporte/context binding y SHA de la matriz 0.8.8.1. Fingerprint: `6ff2ca659cd2db24d246c4cfa321fba1788203e4c956cebddbe161d50c841aed`. Los pins no son una medición criptográfica de todos los archivos del Steam Linux Runtime; fijan metadata y componentes concretos. La copia completa de Proton sí se verifica mediante su inventario de contenido/metadata.

**COMPROBADO:** el estado serializado `QualifiedExecutorPanel` es informativo. Solo una capacidad privada, no deserializable, producida por el self-test en la sesión nativa concede calificación. Reiniciar LXMI la invalida. No existe fallback DirectProton. Cambiar pins, transporte, fuente/build o copia invalida el executor. La copia instalada no se actualiza ni ejecuta; la copia privada debe coincidir con el manifest de 0.8.8.1 `9d0bb73e241bac979afeda526b589fce6f26ee2d5e8822b7aadc710b179421bb`. Durante ejecución solo se permite variar metadata del archivo exacto `dist.lock`, vacío y con UID/GID/mode conservados; no se ignora todo el árbol.

El environment usa la allowlist previa y paths HOME/XDG privados; añade únicamente el digest generado por backend `LXMI_QUALIFIED_CONTEXT_SHA256`. No se copia environment Steam/ZZZ/LXMI completo. La request conserva 0600, nonce, hash, schema, bounded size, edad, rechazo de symlink, consumo del mismo descriptor y cleanup. El contexto privado/real se vincula mediante manifest separado 0600, hash, expiración, nonce real e identidad del prefix. En modo real solo se acepta la ruta AppID 4162040 derivada del Steam root verificado; no un path del usuario.

### Calificación integrada y límites

El self-test crea una sesión nueva en `$XDG_DATA_HOME/lxmi/tests/qualified-entry-v1/<nonce>/`, copia y verifica Proton con el mecanismo descriptor-anchored existente, inicializa un prefix privado y ejecuta dos repeticiones medidas. Reutiliza exactamente el código de ejecución que atendería la entry real; solo cambian contexto autorizado y request. Conserva recibos de mounts/environment propios vinculados al nonce: container `pressure-vessel`, `/usr`, `/run/host/usr`, `/run/host/fonts`, `/run/host/local-fonts` read-only. No se persiste environment completo.

Cada repetición compara los tres hives semánticamente, identidad raíz, metadata y entradas seleccionadas, mappings y directories de fuentes. Solo acepta reglas exactas `StableExpected` / `VolatileExpected` de 0.8.8.1. Fonts `EnvironmentSpecific`, secciones UNKNOWN, nuevos campos/metadatos, mappings distintos o recibos inválidos impiden Qualified. No hay comparación completa del árbol posterior ni prueba de equivalencia universal con Steam. El self-test integrado complementa la matriz anterior con clones equivalentes; no reemplaza esa matriz por una comparación DirectProton contaminada.

Una prueba preliminar nativa pasó antes de añadir comprobaciones finales de mounts y metadata. Se conserva separada en `executor_qualification_0_8_8_2_initial.json`; no acredita por sí sola el código final.

### Gate real: nueva evidencia read-only

**READ-ONLY HOST:** el backup `f3551122468ed68b825a0182922f3be2` conserva integridad **VERIFIED** y restore **VERIFIED OFF-TARGET**. Sin embargo, la comparación completa del source actual con ese backup falla: `recovery_source_current_mismatch`. Esto distingue backup válido de source actual cubierto por ese estado de recuperación. No se atribuye el writer ni se elimina evidencia del intento UnexpectedMutation anterior.

El preflight actual devuelve `NeedsEvidence`, junto con `baseline_not_captured` y `rollback_insufficient_metadata_only` en la evaluación sin promover recuperación. Prefix UID 1000, device 66306, inode 11538172 y build `1790244902 experimental-11.0-20260924-x86_64` permanecen los esperados. Actividad puntual: 0 coincidencias, 0 procesos sin resolver. No es un lock atómico.

**BLOCKED:** no se solicitó/recogió consentimiento, no se abrió una review ejecutable, no se capturó un baseline para normalizar el fallo y no se creó/reemplazó el backup. No hubo helper/Proton/Wine contra el prefix real ni restore. El guard exige simultáneamente executor Qualified, preflight EligibleForReview, recuperación válida, inactividad/build/identidad/protected paths, storage y consentimiento fresco. La UI deshabilita Review/Run con el código de blocker; el backend sigue imponiéndolo independientemente de la UI.

La futura operación cuenta con token de review de 600 s, consumo único y reserva persistente create-new antes de invocar el launcher. Revalida identidad/build/protected paths e inactividad después de la verificación costosa de la copia. Observación de actividad del juego, solo metadata same-UID, marca `GameBecameActive` sin señales al juego. No hay reintento ni restore automático. Estas defensas se verifican offline; la operación real integrada permanece **NO COMPROBADO** por el gate actual.

### HOST TEST final del executor integrado

**QUALIFIED / HOST TEST:** sesión privada `99c0fa4ebb3c178a451622ea98f0886f`. Seed PASS y dos ejecuciones medidas PASS con helper v0.6 inalterado, protocolo 1, nonce/hash/exit correctos, copia Proton validada después, request/context temporales eliminados y recibos correlacionados del container. El código final exige mounts RO y comparación de metadata; esta ejecución sustituye la preliminar para calificar el executor.

| Evidencia medida | Run 1 | Run 2 |
| --- | --- | --- |
| Helper / nonce / hash / exit | PASS | PASS |
| Fonts | 0 cambios semánticos | 0 cambios semánticos |
| Cuatro device ContainerId | VolatileExpected | VolatileExpected |
| NetworkCards parent vacío | StableExpected: omisión de serialización | Sin nueva diferencia |
| Wow6432Node RunServices / RunServicesOnce | StableExpected: creación vacía | Sin nueva diferencia |
| Metadata seleccionada / mappings / identidad raíz | Política calificada satisfecha | Política calificada satisfecha |
| UNKNOWN / EnvironmentSpecific | 0 | 0 |

Artefacto final: [executor_qualification_0_8_8_2.json](../06_Verificacion/evidencias/executor_qualification_0_8_8_2.json), SHA-256 `35aaf59e5cf992faf7ed67d94423c3ef043adf463258c7c4d7215f6c116c0dd8`. [Tres recibos de container](../06_Verificacion/evidencias/executor_container_receipts_0_8_8_2.json) conservan solo environment/mounts allowlisted, sin secretos. El root y los artefactos privados permanecen en almacenamiento LXMI para revisión y limpieza manual; no se borró ni reemplazó el backup real.

**COMPROBADO:** build del transporte con `cargo build -p lxmi-bridge --example qualified_runtime_transport`; staging de ese ELF fijo con permisos 0700 en `$XDG_DATA_HOME/lxmi/helpers/qualified-runtime-transport`. Cada sesión copia y verifica ese transporte; modificarlo requiere un nuevo self-test. No se modifica ni reconstruye el helper Windows.

**COMPROBADO — Tauri nativo:** Scan Steam, calificación privada final, consulta read-only del prefix y lectura de resultados. Executor Qualified / 2 measured runs, private Proton, runtime/wrapper/pressure-vessel/hash visibles. Preflight NeedsEvidence con los tres blockers concretos; Review y Run disabled/sensitive=false verificados por AT-SPI. Historia DirectProton preservada. Sin checkbox/consentimiento ni Run reales; sin browser-only. [Recibo nativo mínimo](../06_Verificacion/evidencias/executor_qualification_0_8_8_2_native.json). Tauri/Vite propios cerrados después; no control de Steam/Wine/juego. No commit/push ni bump de fase completada.
