# LXMI 0.5.2 — Adquisición y autenticidad de paquetes upstream

**Consultas de metadata fijadas:** 2026-09-28T15:09:09Z (ZZMI) y 2026-09-28T15:09:11Z (XXMI Libraries), UTC.  
**Implementación:** `lxmi-xxmi` v0.5.2 consulta GitHub al solicitarlo, vuelve a resolver el tag exacto elegido y conserva la identidad obtenida junto al paquete administrado. Los datos de abajo son una captura reproducible de esa consulta; no son una lista permanente de versiones soportadas.

## Releases fijadas para la validación

| Paquete | Repositorio oficial | Release/tag/ID | Commit del tag | Publicación | Asset | Tamaño | SHA-256 publicado por GitHub |
|---|---|---|---|---|---|---:|---|
| ZZMI (release que LXMI seleccionó al consultar) | [`leotorrez/ZZMI-Package`](https://github.com/leotorrez/ZZMI-Package) | [`v1.5.0`](https://github.com/leotorrez/ZZMI-Package/releases/tag/v1.5.0) · `393483881` | `e59f87047cd405c5db5476d3b1b499574bc43d67` | 2026-09-22T05:54:50Z | `ZZMI-PACKAGE-v1.5.0.zip` · asset `580682882` | 564,722 B | `d0ca0241538f4ea9f2516ee7931dada84f8d386af8b220c82ed55cfbca11a5be` |
| XXMI Libraries | [`SpectrumQT/XXMI-Libs-Package`](https://github.com/SpectrumQT/XXMI-Libs-Package) | [`v1.1.7`](https://github.com/SpectrumQT/XXMI-Libs-Package/releases/tag/v1.1.7) · `387957029` | `6bf6a746a198c82fb34dbd336cac7ad8e0f4ddc9` | 2026-09-13T16:17:55Z | `XXMI-PACKAGE-v1.1.7.zip` · asset `561522389` | 3,525,026 B | `6ba40887a2d1ccd6221e23d06929d4687371aea9e51b01a7400850f811d54c06` |

El release de Libraries publica también `Manifest.json` como asset independiente `561524670` (563 B; SHA-256 de API `07cd8fe0ea0005a3950bd99af2d7be3599de5ff72b22f58d290931dd8a28c285`). El manifest declara firmas para los tres componentes DLL listados más abajo.

La v1.4.5 de ZZMI (release `371126872`, commit `752475bed3a57f14ccf9c72645d35a798942ae6e`) se consultó al inicio del trabajo y permanece en el storage como versión histórica. La consulta de la UI del 2026-09-28 seleccionó v1.5.0; LXMI almacena la identidad exacta y vuelve a solicitar por tag al descargar.

**UPSTREAM VERIFIED:** repositorio, tag, release ID, commit, fecha y assets se obtuvieron de GitHub Releases API y se fijaron aquí. SHA-256 observado al descargar coincidió con el digest publicado en el API. Los hashes son datos de integridad del asset; la autenticidad criptográfica se evalúa separadamente.

## Firma upstream

La implementación del XXMI Launcher consultada se fijó al commit [`d56786b8dacb00c35204bff45ff5b8b83bd8962a`](https://github.com/SpectrumQT/XXMI-Launcher/tree/d56786b8dacb00c35204bff45ff5b8b83bd8962a). Sus fuentes publican claves DER/Base64 distintas para ZZMI y XXMI Libraries y verifican la firma de release sobre los bytes exactos del ZIP. El algoritmo observado es **ECDSA con SHA-256 y curva secp384r1 (P-384)**; la firma se transporta como Base64 de una firma ASN.1 DER.

- [Verificación de firmas upstream](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/utils/security.py)
- [Verificación de asset en Package Manager](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/package_manager.py)
- [Configuración ZZMI](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/model_importers/zzmi_package.py)
- [Configuración de XXMI Libraries](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/migoto_package.py)

LXMI fija esas claves por tipo de paquete y verifica el ZIP antes de extraerlo. La release de Libraries incluye firmas por componente en el `Manifest.json` separado; LXMI compara ese asset con el digest publicado por GitHub y verifica criptográficamente las firmas de `3dmloader.dll`, `d3d11.dll` y `d3dcompiler_47.dll`. `Manifest.json` como documento no lleva una firma independiente: su vínculo al release proviene del HTTPS/API de GitHub y del digest de asset. SHA-256 detecta divergencias, pero por sí solo no acredita identidad del publisher.

Estados que LXMI conserva: `Verified`, `Invalid`, `Missing`, `Unsupported`, `NotChecked`. La ausencia no se convierte en verificación. El campo `official_release_verified` significa que se verificó la firma publicada para el ZIP y que el contenido coincidió con el contrato estructural; para Libraries también exige las tres firmas de componente. No significa compatibilidad de plataforma, instalación o seguridad de ejecución.

## Inventario observado en los assets

Los inventarios se derivan de la extracción validada, no de listas codificadas completas:

- ZZMI v1.5.0 declara versión raw `1.50` en `Core/ZZMI/main.ini`; su release tag conservado es `v1.5.0`. El payload contiene `d3dx.ini`, `Core/ZZMI/`, `Core/Debugger/` y `ShaderFixes/`. Se inventariaron 47 archivos, 1,116,062 bytes.
- La versión histórica v1.4.5 conserva raw `1.45`, con 33 archivos y 1,041,827 bytes.
- XXMI Libraries contiene `3dmloader.dll`, `d3d11.dll` y `d3dcompiler_47.dll`. `Manifest.json` viaja separado del ZIP y se guarda como metadata auxiliar del payload administrado.

La identidad reproducible es la release upstream (`v1.5.0`, `v1.1.7`); el campo raw interno se preserva por separado y no se usa como sustituto del tag. No se ejecutó ningún EXE/DLL/script.

## Mapping y límites del dry-run

La implementación upstream separa `game_folder` de `importer_path`. Instala el payload del importer en la ruta configurable; la integración de Libraries lleva `d3d11.dll` y `d3dcompiler_47.dll` a esa ruta configurable, mientras `3dmloader.dll` permanece en el paquete para el loader. Fuentes fijadas: [modelo de configuración de importer](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/model_importers/model_importer.py) y [despliegue de Libraries](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/migoto_package.py).

LXMI conoce los destinos relativos del payload, pero no el `importer_path` que el usuario configuraría en XXMI Launcher. Por eso el plan almacena `configured_target_root: null`; solo inspecciona como **candidato de comparación** el directorio del ejecutable local de ZZZ. El dry-run lee y calcula hashes, pero ese candidato no es una autorización ni una afirmación de que allí deba instalarse. `apply_allowed=false` y `writes_performed=false`.

## Compatibilidad, distribución y licencias

La publicación ZZMI explica que el paquete está destinado al flujo XXMI Launcher, no a una instalación manual. El release de Libraries también recomienda el launcher. LXMI conserva ese aviso; el import al storage de LXMI sirve para autenticar, inventariar y revisar el plan, no para sustituir el instalador upstream.

La firma autentica bytes de artefacto respecto a la clave fijada; no determina si el asset admite redistribución. El repo ZZMI declara GPL-3.0 y los paquetes de Libraries reúnen componentes con avisos/licencias que deben auditarse por archivo antes de distribuirlos. LXMI mantiene los paquetes en storage privado local y no crea releases ni redistribuye los ZIP. **La auditoría legal/componente por componente sigue pendiente.**

| Capacidad | Estado |
|---|---|
| Release y firma de paquete consultadas | UPSTREAM VERIFIED |
| Bytes ZIP contra SHA-256 de GitHub | COMPROBADO al descargar |
| Firma ECDSA de ZZMI v1.5.0 y Libraries v1.1.7 | CRYPTOGRAPHICALLY VERIFIED al importar |
| Tres firmas DLL del manifest Libraries | CRYPTOGRAPHICALLY VERIFIED al importar |
| Estructura e inventarios de assets reales | COMPROBADO en import aislado |
| Dependency set ZZMI + Libraries | COMPROBADO por dos paquetes gestionados separados |
| Configuración real `importer_path` | DESCONOCIDA; se mantiene configurable |
| ZZZ Steam, Linux/Proton y ejecución con mods | NO COMPROBADO |
| Apply a ZZZ / cambios en Steam o prefix | NO IMPLEMENTADO |
