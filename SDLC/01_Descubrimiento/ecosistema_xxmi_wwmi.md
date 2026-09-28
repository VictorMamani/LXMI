# Ecosistema XXMI / WWMI — revisión 0.5

Consulta: 2026-09-27, America/La_Paz. **UPSTREAM VERIFIED** significa lectura de fuentes y artefactos oficiales; no prueba ejecución ni compatibilidad Linux. No se copió código ni se distribuyeron binarios upstream en el repositorio LXMI.

## Fuentes fijadas

| Componente | Referencia consultada | Resultado |
|---|---|---|
| XXMI Launcher | [commit d56786b](https://github.com/SpectrumQT/XXMI-Launcher/tree/d56786b8dacb00c35204bff45ff5b8b83bd8962a); release observada v2.2.1 | Aplicación que descarga, configura, actualiza y carga importadores. No es el runtime gráfico. |
| WWMI Package | [commit 6474625](https://github.com/SpectrumQT/WWMI-Package/tree/647462518c1916e04dea1de9048f152326960795); [release v1.0.0](https://github.com/SpectrumQT/WWMI-Package/releases/tag/v1.0.0) | Integración específica para Wuthering Waves: INIs, shaders, recursos y guías. Depende del paquete de bibliotecas XXMI. |
| XXMI Libraries | [commit 4a20acd](https://github.com/SpectrumQT/XXMI-Libs-Package/tree/4a20acd20d0b4733dc5e0a61061b4880be2e6306); [release v1.1.7](https://github.com/SpectrumQT/XXMI-Libs-Package/releases/tag/v1.1.7) | Fork de 3Dmigoto; DLLs y manifiesto con versión y declaraciones de firma. |

El launcher enlaza WWMI-Package y su código declara `requirements=['XXMI']`. [WWMI adapter](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/model_importers/wwmi_package.py). El paquete XXMI apunta a XXMI-Libs-Package. [Migoto adapter](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/src/xxmi_launcher/core/packages/migoto_package.py).

## Layout comprobado en los ZIP de release

WWMI-PACKAGE-v1.0.0.zip tiene en su raíz `d3dx.ini`, `Core/`, `Mods/`, `ShaderFixes/` y README. No contiene DLLs. `Core/WWMI/WuWa-Model-Importer.ini` declara `namespace=WWMIv1` y `global $wwmi_version = 1.00`; este último es un valor raw del INI, **no** un tag semver. `d3dx.ini` incluye el núcleo y su target declarado es `Client-Win64-Shipping.exe`.

`WWMI_REQUIRED_FILES` en `lxmi-xxmi/src/package.rs` fija el contrato estructural de INIs, shaders, fuentes y notificaciones cargadas de esa generación. No es una firma de autenticidad ni un intérprete INI completo. Otros layouts/namespaces necesitan revisión y fixtures antes de admitirse. Una versión ausente se representa como desconocida.

XXMI-PACKAGE-v1.1.7.zip contiene `3dmloader.dll`, `d3d11.dll`, `d3dcompiler_47.dll`. `Manifest.json` se publica como **asset separado** de esa release; declara versión y firmas para las tres DLL. Para importar bibliotecas en LXMI se necesita la carpeta extraída con ese manifiesto al lado de las DLL. No se afirma que el ZIP lo incluya. `nvapi64.dll` aparece como posibilidad en el launcher, pero no estaba en este ZIP: no se exige ni se inventa para 0.5.

Hashes calculados durante la investigación (integridad del artefacto observado, no autenticación criptográfica):

| Asset | SHA-256 |
|---|---|
| WWMI-PACKAGE-v1.0.0.zip | `d4150349d2bbb79dea2c5fcd5ce4887b2540b5efbf92f7f30034bd2ceaac7777` |
| XXMI-PACKAGE-v1.1.7.zip | `6ba40887a2d1ccd6221e23d06929d4687371aea9e51b01a7400850f811d54c06` |

## Ubicación y responsabilidad

Upstream mantiene el importador en su carpeta `WWMI/`, fuera del juego, y despliega bibliotecas al importador. No equivale a copiar todo junto al ejecutable. El adaptador WWMI localiza el proceso bajo `Client/Binaries/Win64/Client-Win64-Shipping.exe` y además tiene lógica de configuración del juego. LXMI **no adopta ni ejecuta** esa lógica.

LXMI propone conservar paquetes originales en `packages/xxmi/<digest>/payload` y un futuro despliegue en `runtimes/wwmi`. En el plan actual se enumeran archivos del importador y las DLL D3D seleccionadas hacia ese destino administrado. Loader y Manifest permanecen en el package store. No hay copias al juego, overrides ni mecanismo de carga Linux aprobado. Las configuraciones futuras se describen como pendientes, sin valores inventados.

La detección inspecciona un conjunto finito de ubicaciones: raíz del juego, subcarpeta WWMI, directorio Win64 y futuro runtime administrado. Una instalación externa de XXMI Launcher puede estar en cualquier otra ruta y **no se descarta** por ausencia en esos candidatos. Un conjunto sin manifiesto de procedencia se informa incompleto; la presencia estructural tampoco prueba autenticidad, activación o compatibilidad.

## Licencias y límites de distribución

| Fuente primaria | Verificado | Decisión LXMI |
|---|---|---|
| [XXMI Launcher LICENSE](https://github.com/SpectrumQT/XXMI-Launcher/blob/d56786b8dacb00c35204bff45ff5b8b83bd8962a/LICENSE) | GPL v3 | No copiar código del launcher en este incremento. |
| [WWMI LICENSE](https://github.com/SpectrumQT/WWMI-Package/blob/647462518c1916e04dea1de9048f152326960795/LICENSE) | GPL v3 declarada por el proyecto | No redistribuir payloads. Revisar recursos/componentes por separado antes de distribuir. |
| [XXMI Libraries COPYING](https://github.com/SpectrumQT/XXMI-Libs-Package/blob/4a20acd20d0b4733dc5e0a61061b4880be2e6306/COPYING.txt) | Código 3Dmigoto bajo GPL v3; avisa que shaders de fixes pueden tener titulares/licencias distintas; referencia Deviare GPL v3 | No asumir una licencia uniforme para todo el ZIP. |
| [DirectXTK LICENSE](https://github.com/SpectrumQT/XXMI-Libs-Package/blob/4a20acd20d0b4733dc5e0a61061b4880be2e6306/DirectXTK/LICENSE) | MIT | Avisos deben revisarse si se distribuye una build que lo incluye. |
| [PCRE2 LICENCE](https://github.com/SpectrumQT/XXMI-Libs-Package/blob/4a20acd20d0b4733dc5e0a61061b4880be2e6306/pcre2-10.30/LICENCE) | BSD con condiciones y excepción descritas en el archivo | Inventariar antes de redistribuir. |
| `Dependencies/d3dcompiler_47.dll`, recursos LiberationSans, otros redistribuibles | **NO COMPROBADO:** no se cerró una matriz de procedencia/licencia para cada binario/recurso de release | Redistribución deshabilitada por decisión de arquitectura, no presentada como autorizada. |

No es una auditoría legal completa. ADR-021 elige importación local externa ahora y posible descarga upstream en el futuro, sin empaquetar artefactos de terceros en LXMI. El código desarrollado consume estructuras y metadata; no copia la implementación de XXMI.

## Verificación y cuestiones abiertas

**COMPROBADO:** inspección de ambos ZIP sin ejecutar contenido; directorios extraídos en `/tmp` para contrastar el validador; importación local de ambos en un store de desarrollo; hashes y generación de plan.

**NO COMPROBADO:** firmas upstream, carga bajo Proton, selección de runtime por Steam, activación de WWMI, compatibilidad del juego, licencias de todos los redistribuibles. Los tests del repositorio usan **FIXTURE** de texto sintético, no binarios upstream. El launcher documenta soporte portable vía Wine, pero eso no demuestra que el flujo de LXMI funcione.
