# Primera carga ZZZ: experimento local explícito

Este runner es un experimento opt-in ligado a la instalación de este host:
ZZZ/AppID 4162040 en `/media/university/Steam/SteamLibrary`, Proton Experimental
y Steam Linux Runtime 4 instalados en `~/.local/share/Steam`. No es un executor
de producción ni habilita `launch_with_mods_available` en la aplicación.

El usuario pidió comenzar la carga de mods después de observar el contexto
Steam de la instalación externa. Esta petición se registra por separado del
experimento histórico 0.8.8; sus reservas y consentimientos no se reutilizan.

## Preparar con ZZZ y su wineserver cerrados

Desde `aplicacion`:

```bash
python3 tools/zzz-load-experiment/experiment.py prepare
```

Comprueba el inventario SHA-256 del runtime seleccionado y el hash del loader
oficial que registra su manifiesto. Compila `LXMI ZZZ Loader.exe` con MinGW,
crea una copia de trabajo de ZZMI y copia el árbol `pfx` sin seguir symlinks.
Verifica contenido, enlaces y modos del backup contra el origen, además de
que el origen permanezca estable durante la copia. Conserva metadata auxiliar
de compatdata. **No se ha probado una restauración al prefix real.**

La copia de trabajo cambia `loader` a la identidad del helper, `log_level`
a `warning` y `unbuffered` a `1`. Biblioteca, perfil seleccionado y runtime derivado
se conservan. La sesión registra hashes del helper, código del experimento,
Proton, entrada/runtime Steam, loader y selección. Una preparación incompleta
no contiene `prepared.json` y no se puede ejecutar.

Para aislar un cierre, `prepare --without-mods` crea una copia con `Mods`
vacío y un perfil de diagnóstico de cero mods dentro del plan. Conserva el
perfil seleccionado del usuario y registra su identidad como `source_profile`.
No modifica su biblioteca ni activa una selección diferente en la aplicación.

## Ejecutar una vez mientras el usuario inicia ZZZ desde Steam

Usar la ruta exacta que imprimió la preparación:

```bash
LXMI_CONFIRM_ZZZ_MOD_EXPERIMENT=YES python3 tools/zzz-load-experiment/experiment.py run --session /home/university/.local/share/lxmi/experiments/zzz-direct-ID --wait-seconds 300
```

Es una operación con efectos: inicia un proceso Windows en el contexto del
juego y puede modificar el prefix. El runner espera una única ejecución de
ZZZ con ruta, AppID, compatdata, WINEPREFIX y Proton coincidentes. La consulta
de procfs conserva solo las claves de contexto permitidas; no guarda el entorno
ni argv completos. Se ejecuta mediante Steam Runtime 4 y `proton runinprefix`.

El helper enumera procesos Windows para localizar exclusivamente
`ZenlessZoneZero.exe`, exige un único candidato y verifica la identidad del
archivo del ejecutable y el instante de creación. Resuelve las letras de unidad
de Steam: `S:` y `Z:` pueden representar el mismo archivo. Mantiene abierto el handle de consulta durante la
operación, comprueba que la DLL no esté ya cargada y llama al export `Inject`
de `3dmloader.dll` con el PID identificado y la DLL de la copia de trabajo.
Sin `--hook` no usa Hook (ver la sección final). Nunca usa elevación, debug privileges, PIDs proporcionados por argv,
terminación de juegos ni cambios de opciones Steam.

`consumed.json` se crea sin sobrescritura antes de entrar en Proton. Fallo,
interrupción o timeout consumen esa sesión. No borrar el recibo ni repetir la
sesión. Ante timeout solo se señala al launcher creado por el runner; no se
afirma que una operación Windows pendiente haya sido cancelada. Es necesario
inspeccionar el estado antes de considerar otro experimento.

## Evidencia y límites

### Esperar un arranque nuevo

La primera entrada con el juego ya abierto cargó la DLL pero no produjo cambio
visual. Para comprobar una entrada más temprana se añadió `--arm-before-game`:
preparar una sesión nueva con ZZZ/HoYoPlay cerrados, ejecutar `run` con esa
opción **después de abrir HoYoPlay desde Steam y antes de pulsar Jugar**.
Esperar `armed_waiting_for_new_zzz` en el stdout privado del helper antes de
pulsar Jugar. La preparación y su backup se hacen con ambos cerrados. Esta
modalidad espera hasta 300 segundos y acepta solo
un proceso ZZZ creado después de armarse. Comprueba la identidad del archivo,
espera a que `d3d11.dll`/`dxgi.dll` estén mapeadas y llama al export oficial.
No suspende el proceso: existe una carrera con la creación del dispositivo
gráfico. Registra la edad del proceso al intentar la carga. Antes de entrar
verifica AppID/prefix/Proton del proceso exacto `HYP.exe` de esta instalación,
y después contrasta el contexto Linux del juego. El target Windows se vincula
por la identidad de su archivo y su tiempo de creación.

Steam puede usar `waitforexitandrun`, que espera a que termine un wineserver
anterior. Por ello el helper debe entrar al contexto que Steam ya inició para
HoYoPlay. No armarlo antes de iniciar Steam. Esta modalidad también consume la
sesión aunque no aparezca ZZZ. No repetir un ID consumido.

Cada sesión privada conserva el plan, inventario y backup, contexto observado,
inventarios del prefix antes/después, stdout/stderr y resultado. El código cero
del loader se combina con la presencia del módulo por su ruta exacta en ZZZ.
Eso no acredita que sus hooks gráficos funcionen, que todos los INI se hayan
interpretado ni que un mod sea visible. La comprobación visual del usuario es
un resultado separado. Los cambios naturales de Steam/juego impiden atribuir
todo el diff del prefix al helper.

Después del retorno de Inject, el helper revisado observa durante 15 segundos
el handle del proceso original. Registra su código de salida si termina, sin
seguir por nombre una ejecución nueva ni cargarla automáticamente. Una ventana
superada solo acredita que permaneció vivo esos 15 segundos; la comprobación
visual y estabilidad posterior siguen siendo necesarias. Este cambio responde
al cierre temprano y ausencia de efecto comunicados por el usuario en la
sesión `zzz-direct-15740e8101d0fd0eee6dffd9a5c0a138`.

El runner no cambia `GENERAL_DATA.bin` ni ajusta gráficos. La integración
upstream pide **Character Quality: High** y **High-Precision Character
Animation: Disabled**; revisar esos valores manualmente al comprobar Miyabi.

La vuelta a una ejecución normal requiere cerrar ZZZ por su interfaz y abrirlo
de nuevo desde Steam sin este runner. Esto descarga el módulo con el proceso;
no restaura escrituras del prefix. No hay restore automático ni cambios del
registro para implantar la carga en futuros inicios.

Fuentes primarias consultadas:

- [Direct Inject de XXMI Launcher v2.2.1](https://github.com/SpectrumQT/XXMI-Launcher/blob/v2.2.1/src/xxmi_launcher/core/utils/dll_injector.py).
- [Configuración ZZMI y ajustes gráficos](https://github.com/SpectrumQT/XXMI-Launcher/blob/v2.2.1/src/xxmi_launcher/core/packages/model_importers/zzmi_package.py).
- [Opciones de inicio upstream](https://github.com/SpectrumQT/XXMI-Launcher/wiki/More-Start-Options).

La documentación de Steam en esa wiki describe Windows. El presente runner
es una prueba local de la traducción a Linux, no una afirmación upstream de
compatibilidad con Proton.

## Diagnóstico gráfico sin ejecutar el juego

`graphics_fixture.c` crea una ventana oculta de 64×64 y ejecuta la creación de
un dispositivo/swapchain y un Present. `diagnose_graphics.py` compila ese proceso
propio y lo ejecuta en un prefix copiado bajo `/tmp/lxmi-zzz-graphics-<nonce>`:

```bash
LXMI_CONFIRM_PRIVATE_GRAPHICS_DIAGNOSTIC=YES python3 tools/zzz-load-experiment/diagnose_graphics.py
LXMI_CONFIRM_PRIVATE_GRAPHICS_DIAGNOSTIC=YES python3 tools/zzz-load-experiment/diagnose_graphics.py --hook-paths-only
```

La primera modalidad compara Direct3D nativo, ZZMI sin mods y el perfil
completo; la segunda entra por exports originales interceptados, con creación
de dispositivo/swapchain combinada y separada. Se usa `lxmi-xxmi.dll` como
basename distinto para evitar una resolución ambigua del módulo de sistema y
se verifica su ruta completa antes de contar la prueba como válida. Las copias
de INI usan `log_level = info` y `unbuffered = 1` solo en este fixture acotado.

Los resultados requieren identidad de módulo, log e inicialización completa,
Present y terminación del fixture. Las primeras cinco pruebas válidas dieron
exit 0; los INI de Miyabi/RabbitFX fueron vistos en los logs, sin errores
detectados. No se dibuja una escena de ZZZ ni se prueban sus buffers/hashes,
Unity, su integración real o la apariencia de los mods. El fixture no abre
procesos ajenos, no usa injector ni entra en el prefix activo del juego.

## Hook oficial: fixture de aislamiento y modalidad ZZZ

`hook_fixture.c` + `hook_isolation.py` ejecutan `HookLibrary`/`UnhookLibrary`
del `3dmloader.dll` oficial con una copia del runtime seleccionado, en dos
prefixes Proton nuevos bajo `~/.local/share/lxmi/tests/hook-isolation-<id>/`.
No abre ZZZ, HoYoPlay ni Steam y no entra en el prefix del juego:

```bash
LXMI_CONFIRM_HOOK_ISOLATION_FIXTURE=YES python3 tools/zzz-load-experiment/hook_isolation.py
```

Las imágenes viven en `bin/` y el runtime en `ZZMI/`, como hermanos: upstream
acepta cualquier proceso cuyo directorio de imagen esté contenido en el de la
DLL. `run` de Proton solo inicializa cada prefix (no reenvía stdout); los casos
medidos usan `runinprefix` con los overrides DXVK que `runinprefix` omite.
Cada prefix ocupa varios cientos de MB y se conserva para inspección.

Para ZZZ, tras un `prepare` nuevo con juego y HoYoPlay cerrados y HoYoPlay
abierto desde Steam:

```bash
LXMI_CONFIRM_ZZZ_MOD_EXPERIMENT=YES python3 tools/zzz-load-experiment/experiment.py run --session /home/university/.local/share/lxmi/experiments/zzz-direct-ID --arm-before-game --hook
```

El helper instala el hook antes de que exista ZZZ y lo retira en cuanto ve
ZZMI por ruta exacta en el proceso nuevo, o al agotar el tiempo. Mientras está
instalado, los procesos del mismo wineserver (HoYoPlay incluido) cargan y
rechazan el módulo. La sesión se consume igual que con Inject.
