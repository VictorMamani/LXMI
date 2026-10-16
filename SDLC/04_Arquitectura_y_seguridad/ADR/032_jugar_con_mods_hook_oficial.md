# ADR-032: «Jugar con mods» mediante el Hook oficial de XXMI

- **Estado:** Aceptada para Zenless Zone Zero en Steam + Proton.
- **Fecha:** 2026-10-15.
- **Sustituye:** el `NoDecision` de [ADR-027](027_xxmi_loader_strategy_for_lxmi.md) para ZZMI/ZZZ.

## Contexto

Con `Inject` tardío el módulo se cargaba, pero o bien llegaba después de crear el
dispositivo (sin efecto visual) o el juego se cerraba con `0xc0000409`. El
fixture de aislamiento demostró que el `HookLibrary` oficial queda confinado a su
wineserver y que ZZMI rechaza en `DllMain` todo proceso ajeno. La sesión
consentida `zzz-direct-f8acde73c92b4a6559573efd2831c484` instaló el hook en el
contexto de HoYoPlay, el juego recibió ZZMI antes del dispositivo, siguió vivo y
el usuario vio los mods de Miyabi en partida.

## Decisión

LXMI inicia ZZZ con mods así:

1. Copia de arranque privada del runtime de perfil, verificada contra su
   inventario (3DMigoto escribe log, caché y `d3dx_user.ini` junto a su DLL).
   `[Loader] loader` pasa a `LXMI Loader.exe`; `target` debe ser el ejecutable
   del juego. El helper es el único `.exe` del árbol.
2. Si HoYoPlay no está abierto, `steam -applaunch 4162040`. LXMI identifica el
   proceso exacto de HoYoPlay y exige AppID, compatdata y `WINEPREFIX` del juego.
3. El helper se ejecuta con el Steam Linux Runtime y el Proton que figuran en
   `STEAM_COMPAT_TOOL_PATHS` de HoYoPlay (`proton runinprefix`), con
   `WINEDLLOVERRIDES` de DXVK explícitos. Se pide al servicio de Steam
   (`steam-runtime-launch-client --alongside-steam --clear-env`): en Ubuntu,
   `apparmor_restrict_unprivileged_userns` impide a la app, sin perfil
   AppArmor, crear el contenedor del SLR, y Steam sí tiene perfil. El comando
   anuncia su grupo de procesos para que **Cancelar** lo termine entero; si el
   servicio no responde, se lanza directamente.
4. `HookLibrary` antes de que exista el juego; espera un proceso nuevo cuyo
   archivo coincida por identidad; `UnhookLibrary` en cuanto ve ZZMI por ruta
   exacta en ese PID, o a los 5 minutos; observa 15 s el handle original.
5. El cargador se compila con MinGW en `build.rs` y se incrusta en la app.

No se usan `Inject`, privilegios, suspensión, terminación de procesos ajenos ni
cambios en Steam, el juego o el prefix.

## Consecuencias

- Durante la espera, los procesos del mismo wineserver (HoYoPlay) cargan y
  rechazan la DLL; es el diseño upstream.
- Si el juego resolviera `d3d11` antes de su primera ventana, ZZMI saldría con
  `0xc0000135`; la app lo explica.
- Wuthering Waves sigue bloqueado: upstream usa Direct Inject para WWMI y LXMI
  no lo ha validado.
- Los paneles de diagnóstico 0.8.x conservan su modelo histórico
  (`NoDecision`); no gobiernan este flujo.
- Validado en una combinación (Proton Experimental `experimental-11.0-20261008`,
  SLR 4, ZZMI y Libraries oficiales). Otras requieren nueva verificación.

Ver [verificación](../../06_Verificacion/verificacion_jugar_con_mods.md) y
[manual de uso](../../../MANUAL_DE_USO.md).
