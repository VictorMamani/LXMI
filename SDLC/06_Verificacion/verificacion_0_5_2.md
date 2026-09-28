# Verificación LXMI 0.5.2 — releases oficiales y dry-run

**Fecha de validación:** 2026-09-28  
**Repositorio:** `main`, checkpoint base `386ca5c`; cambios 0.5.2 sin commit.  
**Alcance:** adquirir y validar paquetes, almacenarlos fuera de Steam y comparar el mapping de forma read-only. No hay operación `apply`.

## Releases fijadas

| Paquete | Repositorio/tag | Release ID | Commit resuelto | Asset | SHA-256 publicado por GitHub | Estado |
|---|---|---:|---|---|---|---|
| ZZMI (selección vigente de LXMI) | `leotorrez/ZZMI-Package` · `v1.5.0` | `393483881` | `e59f87047cd405c5db5476d3b1b499574bc43d67` | `ZZMI-PACKAGE-v1.5.0.zip` · 564,722 bytes · asset `580682882` | `d0ca0241538f4ea9f2516ee7931dada84f8d386af8b220c82ed55cfbca11a5be` | UPSTREAM VERIFIED |
| XXMI Libraries | `SpectrumQT/XXMI-Libs-Package` · `v1.1.7` | `387957029` | `6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9` | `XXMI-PACKAGE-v1.1.7.zip` · 3,525,026 bytes | `6ba40887a2d1ccd6221e23d06929d4687371aea9e51b01a7400850f811d54c06` | UPSTREAM VERIFIED |
| XXMI Libraries companion | mismo release | `387957029` | mismo commit | `Manifest.json` · asset `561524670` · 563 bytes | `07cd8fe0ea0005a3950bd99af2d7be3599de5ff72b22f58d290931dd8a28c285` | UPSTREAM VERIFIED |

LXMI consultó la metadata seleccionada en la ventana el 2026-09-28 a las 15:19:16Z (ZZMI) y 15:19:17Z (Libraries); la prueba HTTP fijada la consultó de nuevo a las 15:09:09Z y 15:09:11Z. ZZMI se publicó el `2026-09-22T05:54:50Z`; Libraries el `2026-09-13T16:17:55Z`. Los SHA-256 de las descargas coincidieron con los digests publicados por GitHub.

La v1.4.5 consultada al comenzar también permanece como paquete histórico independiente en el storage; no es la release que la ventana selecciona actualmente. Fuentes upstream: [ZZMI v1.5.0](https://github.com/leotorrez/ZZMI-Package/releases/tag/v1.5.0), [XXMI Libraries v1.1.7](https://github.com/SpectrumQT/XXMI-Libs-Package/releases/tag/v1.1.7), [verificador de firmas del XXMI Launcher en commit fijado](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/utils/security.py) y [gestión upstream de paquetes](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/package_manager.py).

## Autenticidad y storage

- **CRYPTOGRAPHICALLY VERIFIED:** el ZIP seleccionado ZZMI v1.5.0 y el ZIP de Libraries v1.1.7 verificaron con ECDSA P-384/SHA-256 usando las claves públicas fijadas de XXMI Launcher y los bytes exactos firmados. La v1.4.5 histórica también conserva una verificación independiente.
- **CRYPTOGRAPHICALLY VERIFIED:** el `Manifest.json` separado de Libraries coincidió con el digest publicado por la API. Las firmas declaradas para `3dmloader.dll`, `d3d11.dll` y `d3dcompiler_47.dll` verificaron sobre cada archivo real extraído.
- **COMPROBADO:** la prueba HTTP explícita del cliente LXMI descargó ZZMI v1.5.0 y Libraries v1.1.7 mediante streaming HTTPS a un cache y managed store temporales; verificó firmas/layout, importó ambos y dejó tres assets completos (dos ZIP y `Manifest.json`), sin temporales parciales.
- **COMPROBADO:** la UI Tauri descargó e importó ZZMI v1.5.0 a `$XDG_DATA_HOME/lxmi` (`/home/university/.local/share/lxmi`) tras comprobar que el store no se solapa con Steam, ZZZ ni compatdata. La descarga de Libraries v1.1.7 usó su asset oficial y reutilizó el paquete cuyo fingerprint ya estaba gestionado.
- **COMPROBADO:** el managed inventory contiene ZZMI v1.5.0 (47 archivos, 1,116,062 bytes), ZZMI v1.4.5 histórico (33 archivos, 1,041,827 bytes) y XXMI Libraries v1.1.7 (4 archivos, 8,096,499 bytes). IDs SHA-256: `a4336db165e16c184012e0b6c9f72467aa7bdc50dcaa3d74dacb174e4fcfd606`, `2e3d42aaad51d6c59ed1aa6ec1a68abd5fc22a8ea6108477b9159d8a1a103cb6` y `ee6c9955c3684d9c701a1edf2c3fec9806e117fece3a10ef1c5e97c0299a5954`.
- `SHA-256` local detecta alteración del inventario después del import; por sí solo no autentica al publicador. La firma upstream prueba correspondencia con la clave fija, no compatibilidad con ZZZ/Linux ni derechos de redistribución.

## Inventario y mapping revisable

El mapping se deriva del inventario extraído. Cada archivo ZZMI se dirige a la misma ruta relativa bajo el `importer_path` configurable. Desde Libraries, el upstream despliega `d3d11.dll` y `d3dcompiler_47.dll` en ese mismo root; `3dmloader.dll` y `Manifest.json` se mantienen en el paquete gestionado.

El inventario real v1.5.0 tiene 47 archivos ZZMI; el plan añade 2 DLL de Libraries, para 49 mappings:

```text
Core/Debugger/Debugger.ini
Core/Debugger/Fonts/LiberationSans-Bold.dds
Core/Debugger/Fonts/LiberationSans-Bold.png
Core/Debugger/Shaders/Debugger.hlsl
Core/Debugger/Shaders/debug_cb.hlsl
Core/Debugger/debug_cb.ini
Core/ZZMI/FlatNormalMap.dds
Core/ZZMI/Fonts/LiberationSans-Bold.dds
Core/ZZMI/Fonts/LiberationSans-Bold.png
Core/ZZMI/Libraries/HP bar/Includes.ini
Core/ZZMI/Libraries/HP bar/OrderIds.CS.hlsl
Core/ZZMI/Libraries/HP bar/SummHp.CS.hlsl
Core/ZZMI/Libraries/HP bar/characters.ini
Core/ZZMI/Libraries/HP bar/hp.ini
Core/ZZMI/Libraries/Includes.ini
Core/ZZMI/Libraries/SlotFix/Aliases.ini
Core/ZZMI/Libraries/SlotFix/Includes.ini
Core/ZZMI/Libraries/SlotFix/Matches.ini
Core/ZZMI/Libraries/SlotFix/Resources/EmptyGlowMap.dds
Core/ZZMI/Libraries/SlotFix/Resources/FlatNormalMap.dds
Core/ZZMI/Libraries/SlotFix/Resources/Resources.ini
Core/ZZMI/Libraries/SlotFix/SlotFix.ini
Core/ZZMI/Libraries/TTLib/GameHooks.ini
Core/ZZMI/Libraries/TTLib/Includes.ini
Core/ZZMI/Libraries/TTLib/Shaders/CaptureMask.hlsl
Core/ZZMI/Libraries/TTLib/Shaders/ColorCommon.hlsli
Core/ZZMI/Libraries/TTLib/Shaders/LateWorldComposite.hlsl
Core/ZZMI/Libraries/TTLib/Shaders/LateWorldFlipDetect.hlsl
Core/ZZMI/Libraries/TTLib/Shaders/LayerCommon.hlsli
Core/ZZMI/Libraries/TTLib/Shaders/NativeComposite.hlsl
Core/ZZMI/Libraries/TTLib/Shaders/PortraitComposite.hlsl
Core/ZZMI/Libraries/TTLib/Shaders/UIPlacementMeasure.hlsl
Core/ZZMI/Libraries/TTLib/Shaders/UIPlacementPublish.hlsl
Core/ZZMI/Libraries/TTLib/Shaders/portrait_blit_vs.hlsl
Core/ZZMI/Libraries/TTLib/TTLib.ini
Core/ZZMI/Libraries/TTLib/UIPlacement.ini
Core/ZZMI/Notifications/HuntingModeGuide.md
Core/ZZMI/Notifications/UserGuide.md
Core/ZZMI/Shaders/ShapeKeyApply.hlsl
Core/ZZMI/Shaders/ShapeKeyCBOverride.hlsl
Core/ZZMI/Shaders/ShapeKeyMultiplier.hlsl
Core/ZZMI/Shaders/TextPrinter.hlsl
Core/ZZMI/d3dx_patch.ini
Core/ZZMI/help.ini
Core/ZZMI/main.ini
ShaderFixes/Sucrose.png
d3dx.ini
d3d11.dll
d3dcompiler_47.dll
```

**UPSTREAM VERIFIED:** XXMI Launcher separa `game_folder` e `importer_folder`; el import copia el contenido al `importer_path` configurado. La configuración pertenece al usuario y no se conoce automáticamente. Por eso el plan mantiene `configured_target_root = null` y usa únicamente como comparación el directorio real del ejecutable de ZZZ. Ese candidato no es evidencia de que los archivos deban instalarse ahí.

## Dry-run real sobre ZZZ

- **COMPROBADO:** el scan local encontró el manifest Steam `4162040` y el ejecutable `/home/university/.local/share/Steam/steamapps/common/Zenless Zone Zero/games/ZenlessZoneZero Game/ZenlessZoneZero.exe`.
- **COMPROBADO:** la prueba combinó ZZMI v1.5.0 y XXMI Libraries v1.1.7 verificados con el `GameRuntimePlan` local. Produjo 49 mappings y comparó el directorio padre del ejecutable como candidato. Resultado: 49 `WouldCreate`; ningún archivo destino candidato existía.
- Antes y después del dry-run, el test guardó el estado/hash de cada path candidato. Ambos snapshots fueron idénticos. El plan declaró `configured_target_root=null`, `root_is_authoritative=false`, `apply_allowed=false`, `writes_performed=false`, `executable=false`.
- **NO COMPROBADO:** la carpeta configurable real de XXMI `importer_path`; compatibilidad o inyección en Steam/ZZZ/Linux/Proton; lanzamiento del juego con mods.
- No se escribió en Steam, ZZZ, `compatdata/4162040` o `pfx`.

## UI y validación manual

- **COMPROBADO:** `npm run tauri:dev` abrió la ventana nativa. En la UI se ejecutó Scan Steam; apareció ZZZ/AppID `4162040`, ejecutable esperado, compatdata/pfx candidato y runtime Proton seleccionado `No determinado`.
- **COMPROBADO:** desde la ventana se consultaron los releases fijados; se descargó/verificó/importó ZZMI v1.5.0 y XXMI Libraries v1.1.7, se vieron sus paquetes oficiales gestionados y dependencia completa, y se pulsó “Revisar plan de instalación”. La UI mostró 49 destinations, candidato de comparación, `platform_compatibility_unverified`, `apply permitido: no` y `escrituras realizadas: no`.
- **NO COMPROBADO:** el campo de destino real configurable `importer_path`; Steam/Linux/Proton, inyección o ejecución de ZZZ con mods.
- No se presenta compatibilidad Steam/Linux/Proton como verificada. El upstream recomienda su launcher para aplicar los paquetes y LXMI no lo sustituye.

## Comandos y estado

Descarga HTTP real por el proveedor LXMI, aislada en temp y explícita (no forma parte de `cargo test --workspace`):

```bash
LXMI_CONFIRM_OFFICIAL_DOWNLOAD_AND_STORE=YES \
cargo test -p lxmi-xxmi --test upstream_download -- --ignored --nocapture
```

Resultado: 1 test pasó; la API confirmó los release IDs/commits fijados, assets y timestamps; `OfficialReleaseVerified`; caché con 3 assets y sin `.partial`.

Dry-run real de los packages persistidos y ZZZ local, también explícito/ignorado:

```bash
LXMI_CONFIRM_REAL_ZZZ_DRY_RUN=YES \
LXMI_UPSTREAM_STORE_ROOT=/home/university/.local/share/lxmi \
LXMI_REAL_ZZZ_EXECUTABLE='/home/university/.local/share/Steam/steamapps/common/Zenless Zone Zero/games/ZenlessZoneZero Game/ZenlessZoneZero.exe' \
cargo test -p lxmi-xxmi --test upstream_real dry_runs_current_official_zzmi_and_libraries_against_local_zzz_read_only -- --ignored --nocapture
```

Resultado: 1 test pasó; 49 `WouldCreate`, `configured_target_root=null`, `apply_allowed=false`, `writes_performed=false`; los snapshots de cada destino candidato fueron iguales antes y después. Una prueba inicial anterior detectó la lectura por `/proc/self/fd` incompatible con el descriptor seguro; se corrigió la verificación de DLL para usar el `Directory` ya abierto.

## Gates finales

- `cargo fmt --all -- --check`: pasó.
- `cargo check --workspace`: pasó.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: pasó.
- `cargo test --workspace`: pasó; 140 pruebas aprobadas, 4 ignoradas por requerir Steam local o una acción explícita de red/dry-run.
- En `apps/lxmi-desktop`: `npm run typecheck`, `npm run lint`, `npm run format:check` y `npm run build`: pasaron.
- `git diff --check`: pasó.
- `npm run tauri:dev`: la ventana nativa se abrió y se recorrió manualmente el flujo indicado arriba.
