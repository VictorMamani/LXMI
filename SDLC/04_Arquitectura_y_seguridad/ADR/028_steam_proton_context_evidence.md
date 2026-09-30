# ADR-028 — Evidencia de contexto Steam / Proton

- **Estado:** Aceptada para LXMI 0.8.2
- **Fecha:** 2026-09-30
- **Ámbito:** discovery y readiness read-only de Zenless Zone Zero / Steam AppID `4162040`

## Contexto

LXMI detecta compatibility tools instaladas, manifest de juego, `compatdata/<AppID>` y `pfx`. Estos hechos no identifican por sí solos la herramienta que Steam usó para ejecutar el juego. La selección configurada tampoco demuestra el contexto efectivo de una ejecución concreta.

## Decisión

`lxmi-steam` inspecciona de forma read-only `config/config.vdf` desde el Steam root asociado con la biblioteca del manifest. Reutiliza Valve KeyValues con límite de tamaño y valida componentes de ruta. El mapping por AppID y la entrada global `0` se conservan en campos distintos. La herramienta configurada puede relacionarse con la lista de tools instaladas por su internal ID, pero no se copia a `GameRuntimePlan.selected` ni a `effective_runtime_selection`.

`compatdata`, `pfx`, runtime efectivo, prefix efectivo, contexto de proceso, relación con wineserver y requisitos same-prefix/same-wineserver son observaciones/capacidades separadas. En ausencia de observación de proceso, las relaciones efectivas permanecen `Unknown`. La readiness sigue `NeedsEvidence`, `LoaderStrategyDecision` sigue `NoDecision` y el gate no adquiere executor.

## Alternativas descartadas

- Elegir el único Proton instalado: la disponibilidad no demuestra selección.
- Elevar un mapping global `0` a selección de ZZZ: su alcance no equivale a una entrada por AppID y no se observó el proceso efectivo.
- Tratar la existencia de `compatdata/4162040/pfx` como prueba de runtime o prefix actualmente usado.
- Iniciar Steam/Proton/ZZZ o inspeccionar procesos para resolverlo: fuera del alcance read-only de 0.8.2.

## Consecuencias

La UI puede mostrar datos configurados, detected candidates y contexto desconocido de forma explícita. Cuando Steam no registra un override en el archivo inspeccionado, LXMI informa esa ausencia junto a la fuente consultada; no concluye cuál runtime se usará.

En el host consultado, el bloque `CompatToolMapping` de `config/config.vdf` estaba vacío; `compatdata/4162040` y `pfx` existían; Proton Experimental estaba instalado. El runtime efectivo, prefix efectivo, wineserver y requisitos del loader siguen sin determinar.

## Referencias

- [Evidencia Steam/Proton 0.8.2](../../01_Descubrimiento/evidencia_contexto_steam_proton_0_8_2.md)
- [Verificación LXMI 0.8.2](../../06_Verificacion/verificacion_0_8_2.md)
- [ADR-027 — Loader Strategy](027_xxmi_loader_strategy_for_lxmi.md)
