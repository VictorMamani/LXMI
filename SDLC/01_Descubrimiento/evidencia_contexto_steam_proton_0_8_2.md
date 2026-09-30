# LXMI 0.8.2 — Evidencia del contexto de procesos Steam / Proton

**Consulta local:** 2026-09-30. Alcance de solo lectura. No se iniciaron Steam, Proton, Wine, wineserver, ZZZ ni loaders.

## Hallazgos

### Steam compatibility-tool selection

- **LOCAL VERIFIED:** el Steam root detectado es `/home/university/.local/share/Steam`. Su `config/config.vdf` contiene la sección `InstallConfigStore/Software/Valve/Steam/CompatToolMapping`, pero está vacía. No hay un mapping por AppID `4162040` ni un mapping global `0` en ese archivo.
- **LOCAL VERIFIED:** se buscaron las claves `CompatToolMapping` en los dos `userdata/*/config/localconfig.vdf` presentes; no se encontraron.
- **LOCAL VERIFIED:** existe una instalación Proton Experimental con el entrypoint `steamapps/common/Proton - Experimental/proton`. Esto acredita una instalación candidata, no una selección de Steam para ZZZ.
- **UNKNOWN:** qué herramienta ejecutaría efectivamente ZZZ. Un mapping vacío/no encontrado no demuestra que Steam use Proton Experimental ni identifica el comportamiento efectivo de Steam en una futura sesión.
- **UPSTREAM-HOSTED COMMUNITY REPORT:** el reporte [ValveSoftware/steam-for-linux #10184](https://github.com/ValveSoftware/steam-for-linux/issues/10184) describe `CompatToolMapping` en `config.vdf`, entradas por AppID y la entrada `0` como mapping global. Es una observación de un issue, no una especificación de Valve. LXMI por ello reporta el contenido leído como configuración registrada y no como runtime efectivo.

### AppID, biblioteca, compatdata y prefix

- **LOCAL VERIFIED:** Steam encontró `appmanifest_4162040.acf` en `/home/university/.local/share/Steam/steamapps/`.
- **LOCAL VERIFIED:** el ejecutable esperado existe en `/home/university/.local/share/Steam/steamapps/common/Zenless Zone Zero/games/ZenlessZoneZero Game/ZenlessZoneZero.exe`.
- **LOCAL VERIFIED:** existe `/home/university/.local/share/Steam/steamapps/compatdata/4162040/` y también su subdirectorio `pfx/`. LXMI los clasifica como compatdata y prefix candidatos observados.
- **UPSTREAM VERIFIED:** la FAQ de Proton describe prefixes por juego bajo `steamapps/compatdata/<APPID>/pfx/` ([Valve Proton FAQ, snapshot fijado](https://github.com/ValveSoftware/Proton/wiki/Proton-FAQ/9a1684b9b948a44c8b7fda0c32d2fa50f8956bb4)). Esta relación estructural no acredita salud del prefix ni una ejecución activa.
- **INFERRED:** el manifest y el AppID local enlazan el juego con su biblioteca Steam; el árbol `compatdata/4162040/pfx` corresponde al espacio esperado por AppID. La detección pasiva no prueba que Steam lo haya usado recientemente.

### Procesos y límites de lo conocido

- **UPSTREAM VERIFIED:** el código público de Proton usa `STEAM_COMPAT_DATA_PATH` como base de compatdata y define el subdirectorio `pfx` dentro de esa base ([ValveSoftware/Proton `proton_11.0`](https://github.com/ValveSoftware/Proton/blob/proton_11.0/proton)). Es evidencia del contrato del wrapper de Proton, no del proceso efectivo elegido para esta instalación de ZZZ.
- **UNKNOWN:** runtime Proton efectivo, `WINEPREFIX` efectivo, wineserver activo, desktop/session Wine efectivo y visibilidad mutua de procesos.
- **UNKNOWN:** si un futuro loader tendría que compartir el prefix, wineserver o build Proton con ZZZ.
- **ARCHITECTURAL DECISION:** no inspeccionar procesos para inferir esos vínculos ni ejecutar una prueba en `compatdata/4162040`. Mantener `LoaderStrategyDecision = NoDecision`, `GameRuntimePlan.selected = Unknown`, readiness `NeedsEvidence` y el gate no elegible.

## Modelo 0.8.2

`SteamProtonProcessContextEvidence` separa:

- mapping por AppID, mapping global y las herramientas instaladas;
- `compatdata` y `pfx` detectados de un prefix efectivo;
- mapping configurado de runtime efectivo;
- relación de procesos observada de requisitos de prefix/wineserver;
- evidencia con su fuente y estados de lectura/configuración/ausencia/desconocimiento.

La lectura de `config/config.vdf` está limitada a 2 MiB, requiere un archivo regular y rechaza enlaces simbólicos en `config/` y `config.vdf`. La lógica reutiliza el parser Valve KeyValues de `lxmi-steam`; no escribe Steam.

Una entrada global `0` se conserva como dato separado y nunca se promueve al mapping de ZZZ. Ni un solo runtime instalado ni un único candidato se convierte en `selected_game_runtime`.
