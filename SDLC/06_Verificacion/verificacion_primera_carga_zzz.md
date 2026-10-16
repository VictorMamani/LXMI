# Primera carga experimental de ZZMI en ZZZ

Fecha local: 2026-10-15. El usuario pidió comenzar la carga de mods y confirmó
que abriría ZZZ desde Steam. Se creó un experimento separado de la reserva
histórica 0.8.8. No se reutilizó su consentimiento ni se habilitó el executor
de producción de LXMI. La versión permanece 0.8.7.

## Resultado observado

**DLL cargada; sin cambio visual según el usuario.** El helper limitado a ZZZ
devolvió `Inject = 0`, terminó con exit code 0 y encontró la DLL de la copia
de trabajo por su ruta exacta en la lista de módulos del juego. La misma DLL
no estaba presente antes de la llamada. Eso acredita la carga del módulo;
no demuestra interpretación de los INI ni hooks gráficos. El usuario comprobó
a Miyabi y confirmó que su apariencia seguía igual. No se registra éxito de
la carga de mods.

El juego fue abierto manualmente por el usuario mediante Steam. Se verificó
AppID 4162040, compatdata externo, WINEPREFIX y Proton Experimental. La versión
registrada es `1791480010 experimental-11.0-20261008-x86_64`. El helper entró
mediante Steam Linux Runtime 4 y `proton runinprefix`, identificó un único
proceso Windows de ZZZ y comparó la identidad del archivo del ejecutable y su
tiempo de creación antes de llamar al export oficial `Inject`.

Steam presentó el ejecutable como `S:\steamapps\common\Zenless Zone Zero\games\ZenlessZoneZero Game\ZenlessZoneZero.exe`.
La unidad `s:` apunta a `/media/university/Steam/SteamLibrary`; `z:` apunta a
`/`. La primera comprobación de argv exigía la representación `Z:` y se detuvo
**antes de ejecutar Proton/helper/loader**. Se corrigió la normalización de
unidades y la barra final de WINEPREFIX, se recompiló el helper y se conservaron
el plan anterior y su enmienda antes de consumir la sesión. Hubo una sola
llamada real al cargador.

## Preparación y conservación

- Perfil seleccionado: **ZZZ — Miyabi y RabbitFX**, dos mods.
- Inventario del runtime seleccionado: 90 archivos, todos con SHA-256
  coincidente antes y después de la prueba.
- Copia del prefix actual con el juego cerrado: 2402 entradas y 415857746
  bytes regulares. Contenido, enlaces y modos se contrastaron contra el origen
  estable. No se hizo restore al prefix real ni se califica esa recuperación
  como probada.
- La copia de ejecución ajustó `loader = LXMI ZZZ Loader.exe` y
  `log_level = warning`. Conserva los ajustes originales del perfil y el patch
  de Miyabi previamente deshabilitado.
- El recibo de consumo se creó sin sobrescritura antes de ejecutar Proton.
  La sesión no se puede repetir con el runner.

En la captura inmediata del prefix antes/después no hubo altas ni bajas; se
modificaron dos archivos de logs/telemetría de HoYoPlay. No se atribuyen esas
escrituras al helper: el juego y launcher estaban ejecutándose. Tampoco se
afirma que esa captura cubra escrituras diferidas posteriores.

stderr del helper contiene mensajes de Fontconfig sobre la carga de
`/etc/fonts/fonts.conf` y memoria. No impidieron este exit 0 y la observación
del módulo; se conservan como diagnóstico del entorno. No apareció un
`d3d11_log.txt` en las ubicaciones de runtime/juego inspeccionadas. La presencia
del módulo no se presenta como inicialización gráfica completa.

## Artefactos y reproducción

[Runner y límites](../05_Construccion/aplicacion/tools/zzz-load-experiment/README.md).
[Resumen estructurado](evidencias/primera_carga_zzz.json).

Sesión privada: `~/.local/share/lxmi/experiments/zzz-direct-a80848d957426dc899191f57495523dd/`.
Conserva backup, manifiestos, hashes de build, contexto allowlist, stdout/stderr,
recibo y resultado. El resumen durable omite PIDs y entorno completo.

El helper compiló con `-Wall -Wextra -Werror`. La comprobación de esta sesión
es la ejecución real descrita, no una suite sintética ni una prueba de UI.
No hubo commit ni push.

La revisión posterior de [DLLMainHook.cpp v1.1.7](https://github.com/SpectrumQT/XXMI-Libs-Package/blob/v1.1.7/DirectX11/DLLMainHook.cpp)
y [D3D11Wrapper.cpp v1.1.7](https://github.com/SpectrumQT/XXMI-Libs-Package/blob/v1.1.7/DirectX11/D3D11Wrapper.cpp)
encontró interceptación de funciones de creación de dispositivo y carga de
configuración en su inicialización. La carga tardía es una hipótesis para la
ausencia de efecto, no una causa demostrada.

El usuario cerró el juego. Se preparó otra sesión independiente,
`zzz-direct-1234fb6a529fcacfeb211d3629d30b9e`, con un backup nuevo de 2401
entradas / 416589424 bytes, y se armó un helper antes del siguiente inicio
manual. Esta modalidad acepta solo procesos creados después de que el helper
comienza a esperar. Espera las bibliotecas `d3d11.dll` y `dxgi.dll`, que el
DllMain upstream necesita para instalar sus interceptores; no suspende el
juego ni asegura haber llegado antes de la primera creación del dispositivo.
El contexto Steam de ese proceso se contrasta además después de la entrada.
El helper emitió `armed_waiting_for_new_zzz`. El usuario informó de una
actualización de 2 GB en Steam y no inició el juego durante la ventana. Tras
300 segundos, el helper salió con `exact_target_not_visible`, exit 1 y **cero
llamadas a Inject**. No hubo segunda carga ni comprobación visual en esa sesión.

La siguiente modalidad requiere abrir primero HoYoPlay desde Steam y verificar
su contexto antes de armar el helper, para evitar que un `waitforexitandrun`
de Steam espere una sesión Wine creada por LXMI. La revisión de esa secuencia
es preventiva: en la ventana anterior no se observó un Proton de Steam
esperando; la descarga explica la ausencia del target.

Pendiente de ese checkpoint: terminar la actualización, preparar un backup nuevo con juego y
launcher cerrados, abrir HoYoPlay desde Steam, armar el helper y entonces
pulsar Jugar. Esta secuencia revisada aún no se ha ejecutado.

La carga automática desde la aplicación y el cambio de mods sin recarga siguen
fuera de este runner experimental.

## Continuación tras completar la actualización Steam

El usuario confirmó que la actualización terminó. Se preparó la sesión
`zzz-direct-15740e8101d0fd0eee6dffd9a5c0a138`, con juego y HoYoPlay cerrados.
El backup actual tiene 2417 entradas / 417077974 bytes regulares; contenido,
enlaces y modos verificados contra el origen estable. Se añadieron al plan
los hashes del ejecutable actualizado de ZZZ y de `HYP.exe`. La comprobación
de cierre ahora normaliza también la barra final de WINEPREFIX al buscar
wineservers y exige que el launcher esté cerrado durante el backup.

Se abrió HoYoPlay manualmente desde Steam, se verificaron su ejecutable exacto,
AppID 4162040, Proton Experimental y prefix externo y se armó el helper dentro
de ese contexto. Tras recibir `armed_waiting_for_new_zzz`, el usuario inició el
juego desde HoYoPlay. Se identificó un proceso nuevo por su archivo y fecha de
creación. `d3d11.dll`/`dxgi.dll` se observaron mapeadas a los **2797 ms** de vida
del proceso; hubo una llamada a Inject, retorno 0, exit 0 y módulo exacto
presente. La observación posterior volvió a verificar el contexto Steam del
juego. No se suspendió el proceso.

Esta vez apareció `ZZMI/d3d11_log.txt` en la copia de trabajo, con el encabezado
`D3D11 DLL starting init`, ruta de juego `S:` y módulo LXMI por `Z:`. En la
lectura inicial tiene 290 bytes. Esto demuestra que se alcanzó el inicio de
configuración gráfica; no acredita por sí solo que haya terminado ni el efecto
de todos los INI. En el checkpoint inicial la comprobación visual estaba pendiente;
el resultado confirmado posteriormente se detalla a continuación.

Los 90 archivos del runtime seleccionado original conservan sus hashes. La
captura inmediata del prefix registró tres altas, una baja y 35 modificaciones,
con Steam/HoYoPlay/juego activos; no se atribuye ese conjunto al helper ni se
cubren escrituras diferidas. Los diagnósticos Fontconfig reaparecieron en
stderr, sin impedir la carga observada. La versión sigue en 0.8.7 y el gate
de producción continúa cerrado.

[Resumen estructurado de la carga temprana](evidencias/carga_temprana_zzz_20261015.json).

## Resultado visual negativo y terminación de la ejecución inicial

El usuario informó que Miyabi seguía igual y recordó una pantalla negra con
cierre rápido al primer intento; después volvió a abrir ZZZ. Una consulta
pasiva al host confirmó que el proceso al que se cargó ZZMI ya no existía, que
había otro proceso de ZZZ y que el nuevo no mapeaba la DLL del experimento.
El helper de esa sesión ya había terminado: no volvió a inyectar en la nueva
ejecución. El arranque posterior sin esa DLL no demuestra el funcionamiento
ni la ausencia de funcionamiento de ZZMI en esa ejecución.

`Player-prev.log` termina durante la inicialización Direct3D 11; su timestamp
coincide con la entrada temprana de ZZMI. El `Player.log` posterior es más
largo y corresponde al siguiente arranque. La consulta del journal en ese
intervalo no encontró un evento de segfault/coredump/reset de GPU. El log de
ZZMI sigue limitado a 290 bytes. No hay excepción ni stack que permitan
atribuir de forma concluyente el cierre a una causa específica.

**Resultado de carga de mods: no satisfactorio.** La evidencia positiva sigue
limitada a Inject 0, presencia inicial de la DLL y comienzo de configuración;
no acredita estabilidad del juego ni inicialización gráfica completa.

Se añadió una observación de 15 segundos usando el handle del proceso Windows
original después de Inject. Si ese proceso termina, el helper registra su
código de salida; el runner devuelve `module_loaded_original_game_exited`.
Si permanece vivo, registra la ventana observada y conserva la comprobación
visual pendiente. No se busca por nombre un proceso sustituto ni se reinyecta.
Este helper revisado aún no se ha ejecutado contra ZZZ.

Para investigar sin repetir el cierre del juego se creó un fixture gráfico
propio con una ventana oculta y un prefix copiado bajo `/tmp`. Compara un
dispositivo/swapchain/present nativo, ZZMI sin mods y el perfil completo. No
abre procesos del juego, no usa un injector y no entra en su prefix activo.
La primera pasada generó exit 0 en los tres casos pero no logs del wrapper;
no se considera prueba válida de inicialización de ZZMI. Se añadió identidad
exacta del módulo y un basename de DLL propio para verificar la ruta real.

## Resultado del diagnóstico gráfico aislado

La pasada corregida completó Direct3D nativo, ZZMI sin mods y ZZMI con el
perfil completo. En los dos casos de wrapper se verificó su módulo exacto,
el log registró `D3D11 DLL successfully initialized`, se completó el Present y
el proceso terminó con exit 0. El perfil completo cargó los INI de Miyabi y
RabbitFX, sin líneas de error detectadas. La recarga de configuración quedó
registrada; no se probaron sus reemplazos sobre una escena del juego.

La comprobación posterior usó exports originales interceptados por ZZMI,
con creación combinada de dispositivo/swapchain y creación separada mediante
`D3D11CreateDevice`, factory DXGI y `CreateSwapChain`. Ambos casos completaron
inicialización, recarga, Present y exit 0. Se conservaron el runtime original
y la copia de recuperación. Las cinco pruebas válidas están resumidas en
[diagnóstico gráfico](evidencias/diagnostico_grafico_zzz_20261015.json).

Estas pruebas no reproducen el cierre. Son un fixture propio, sin Unity,
escena del juego ni inyección remota; tampoco demuestran que los hashes del
mod correspondan a la versión actual de ZZZ. No se establece la causa del
cierre real.

Se preparó una prueba del juego con `Mods` vacío en la sesión
`zzz-direct-5acc7aa9c0946c51352ed20431ec00bc`. Backup nuevo: 2420 entradas /
417600416 bytes. Mantiene Libraries v1.1.7 y la integración existente para
aislar la presencia de mods. Incorpora el helper que observa 15 segundos al
proceso original. El plan usa el nombre `ZZMI — diagnóstico sin mods` y
conserva el perfil seleccionado como origen, sin cambiar la selección guardada.
En este checkpoint está esperando que el usuario abra HoYoPlay desde Steam;
todavía no se ha ejecutado el helper ni consumido la sesión.

La espera de HoYoPlay terminó sin observar el launcher. La sesión sin mods
permanece sin consumir: no se ejecutó helper ni loader. Tras la interrupción
se confirmó que el proceso de espera había terminado; no queda una carga
armada por ese runner. La prueba real sin mods sigue pendiente.

## Continuaciones registradas y control sin inyección — 2026-10-15

Los pendientes del checkpoint anterior fueron sustituidos por las evidencias
posteriores. La [prueba sin mods con Libraries 1.1.7](evidencias/zzz_sin_mods_20261015.json)
y la [prueba con Libraries 1.2.2](evidencias/zzz_sin_mods_libraries_1_2_2.json)
cargaron el módulo, pero el proceso original terminó antes de completar los
15 segundos de observación, con salida `0xc0000409` en ambos casos.

La [sesión con registro Proton](evidencias/zzz_fallo_registro_proton_20261015.json)
capturó primero `0xc0000005`, una violación de acceso en `GameAssembly.dll`
en el offset relativo `0x27d4d7`, y después la terminación `0xc0000409`.
La causa específica sigue sin establecerse.

Al recuperar el contexto se encontró el resultado de una prueba de control
en `/tmp/lxmi-zzz-debug/normal-game-result.json`: arranque manual mediante
Steam y HoYoPlay, sin helper ni injector, con el mismo proceso vivo durante
30 segundos y sin módulo del experimento LXMI observado. La captura privada
del log tiene 433191 bytes; su tamaño y SHA-256 se verificaron contra el
resultado guardado. No contiene `c0000005`, `c0000409` ni
`EXCEPTION_ACCESS_VIOLATION` en esa ventana.

Este contraste orienta la investigación al flujo de carga de ZZMI, sin
demostrar la causa ni que todas las condiciones de las sesiones fueran
idénticas. El usuario confirmó después que llegó a la pantalla «Pulsa para
entrar» en la prueba sin inyección. No se confirma entrada a la partida ni
estabilidad prolongada. No se inició el juego ni se ejecutó otra inyección
durante esta recuperación de contexto.

[Resumen del control sin inyección](evidencias/zzz_control_sin_inyeccion_20261015.json).
Los logs crudos se conservan privados y no se exportaron al repositorio.

## Revisión del fallo y aislamiento de Hook en fixture — 2026-10-15

Sin iniciar ZZZ ni entrar en su prefix. Se revisaron las copias privadas de
la sesión `zzz-direct-c036a0271c371b6db1da0809b439da08`:

- `Player-prev.log` del arranque fallido termina en `<RI> Input initialized.`;
  en el arranque normal posterior la siguiente línea es
  `<RI> Initialized touch support.`. El fallo ocurre justo después de crear el
  dispositivo gráfico.
- En el log Proton, `ZZMI\d3d11.dll` se carga en el hilo remoto creado por
  `Inject`; a continuación DXVK crea el dispositivo y después el hilo principal
  del juego sufre `0xc0000005`. Tras un handler a `+0xc96000` de la base de
  `GameAssembly.dll` se eleva `0xc0000409`. No se registró el tamaño del módulo
  ni se conservó un dump de Unity. Es **compatible** con una carrera entre la
  carga tardía y la creación del dispositivo; no prueba la causa.
- Los artefactos crudos del control sin inyección en `/tmp/lxmi-zzz-debug/`
  ya no existen; solo quedan tamaño y SHA-256 registrados. La comparación
  completa exige repetir el control.
- Las opciones de lanzamiento de diagnóstico siguen activas en Steam para
  4162040: `PROTON_LOG=1 PROTON_LOG_DIR=/tmp/lxmi-zzz-debug WINEDEBUG=+seh,+loaddll %command%`.
  LXMI no las modifica; quitarlas o mantenerlas es decisión del usuario.

### Fixture de aislamiento Hook

`hook_isolation.py` ejecuta el `HookLibrary` oficial con el runtime ZZMI
seleccionado (copia, `Mods` vacío) en dos prefixes Proton nuevos propiedad de
LXMI. Resultado final (`hook-isolation-ca9df6517014d7ad8a11bdb89843a126`):

| Caso | Resultado |
|---|---|
| `HookLibrary` / `UnhookLibrary` | 0 / 0 |
| Proceso GUI ajeno (sustituto de HoYoPlay) | ZZMI no queda mapeado; exit 0 |
| Procesos Wine del prefix A | Cargan y descargan ZZMI en cada evento CBT (rechazo en DllMain) |
| Mismo target en prefix B con el hook activo en A | Ninguna carga; dispositivo nativo |
| Target que resuelve `d3d11` después de su ventana (2/2) | ZZMI presente antes del dispositivo; vtable del dispositivo en ZZMI; log `successfully initialized`; Present y exit 0 |
| Target que resuelve `d3d11` antes de su ventana (2/2) | `BUG: Unwrapped ID3D11Device!` y salida deliberada `0xc0000135` de upstream |
| Target después de `UnhookLibrary` | Sin ZZMI; dispositivo nativo |

Hallazgos de implementación:

- `proton runinprefix` omite `setup_prefix`, donde Proton añade los overrides
  nativos de DXVK. Sin ellos, `HookLibrary` resolvió `ZZMI\d3d11.dll` como
  `d3d11` builtin de Wine y devolvió 300. El fixture y el runner fijan
  `WINEDLLOVERRIDES=d3d11,d3d10core,d3d9,dxgi,d3d12,d3d12core=n`.
- Upstream acepta cualquier proceso cuyo directorio de imagen esté contenido
  en el directorio de la DLL. El runtime no debe estar bajo un directorio que
  contenga ejecutables ajenos.
- Si ZZZ resolviera `d3d11` antes de su primer evento CBT, el resultado
  esperable es el mismo cierre deliberado `0xc0000135`, distinto del
  `0xc0000409` observado con Inject.

Esto levanta el bloqueo de aislamiento **solo para el fixture**. No acredita
compatibilidad con ZZZ ni efecto visual. [Resumen](evidencias/aislamiento_hook_fixture_20261015.json).

### Siguiente sesión con el usuario (preparada, no ejecutada)

`experiment.py run --arm-before-game --hook` instala el hook antes de que
exista ZZZ, espera un proceso nuevo, verifica ZZMI por ruta exacta en ese PID,
llama `UnhookLibrary` en cuanto lo ve (o al agotar el tiempo) y observa el
proceso original 15 segundos. Requiere una sesión nueva de `prepare` con juego
y HoYoPlay cerrados, abrir HoYoPlay desde Steam y no pulsar Jugar hasta ver
`armed_hook_waiting_for_new_zzz`. No se ha ejecutado contra ZZZ.
