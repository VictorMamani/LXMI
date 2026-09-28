# ADR-025: Native LXMI y puente al runtime Windows XXMI

- **Estado:** aceptada como límite arquitectónico; estrategia de loader diferida.
- **Fecha:** 2026-09-28.
- **Contexto:** LXMI es un proceso nativo Linux; ZZMI y los componentes XXMI descritos upstream son un runtime Windows. `d3dx.ini` conserva la identidad `XXMI Launcher.exe` como nombre de proceso permitido por 3Dmigoto.

## Evidencia

La investigación fijada a `SpectrumQT/XXMI-Launcher` v2.2.1, commit `d56786b8dacb00c35204bff45ff5b8b83bd8962a`, indica que el launcher resuelve `importer_path` relativo a `App.Root`, y ZZMI usa `ZZMI/`. El código actual del paquete Migoto pasa `3dmloader.dll` a `DllInjector` y despliega `d3d11.dll`/`d3dcompiler_47.dll` bajo el importer. El paquete ZZMI se distribuye como integración aparte de XXMI Libraries.

Upstream documenta su launcher portable Linux mediante Wine 9.22+ y Microsoft Visual C++ Redistributable. Eso no verifica el camino ZZZ Steam + Proton. La wiki upstream documenta Native Steam Launch para ZZZ; Direct Steam Launch sigue descrito como pendiente. La exigencia de que un helper comparta el prefix del juego no está determinada.

## Opciones evaluadas

| Opción | Mantenimiento | Licencia | Integración Linux | Estado |
|---|---|---|---|---|
| Ejecutar XXMI Launcher Portable mediante Wine | Máxima fidelidad al flujo upstream; mantiene UI/funciones que LXMI pretende reemplazar | Launcher declara GPL-3.0; redistribución de dependencias y binarios requiere revisión | Existe documentación portable, pero no prueba ZZZ Steam/Proton | No seleccionada |
| Reutilizar componentes mínimos del loader upstream | Podría reducir código nuevo, pero necesita comprender el contrato del proceso y de `3dmloader.dll` | La licencia del repositorio no basta para resolver permisos de cada binario/asset | El loader y el juego tendrían que interoperar con Proton; prefix y rutas siguen desconocidos | No seleccionada |
| Escribir un helper Windows de LXMI | Permite diseñar un futuro canal IPC; aumenta superficie y responsabilidad de ejecución | Código nuevo podría licenciarse separadamente, pero la integración con componentes GPL/terceros debe revisarse | Necesitaría transporte, descubrimiento del prefix y pruebas de interoperabilidad | No seleccionada |

## Decisión

LXMI mantiene un **control plane nativo** que genera datos de runtime/launch topology y administra un `App.Root` privado por XDG. No selecciona launcher, `3dmloader.exe`, `3dmloader.dll` como estrategia de ejecución, ni helper propio. El campo `LoaderStrategy` queda `Unknown`; `SamePrefixRequirement` queda `Unknown`.

Se preserva la posibilidad de un helper/IPC futuro en el modelo de topología, pero no se reserva un protocolo ni se ejecuta ningún componente. Esta decisión evita tratar `LXMI` como sustituto automático del nombre de proceso `XXMI Launcher.exe` y evita afirmar compatibilidad que no se ha probado.

## Consecuencias y verificación requerida

- **Permitido en 0.5.3:** revisar el plan, crear el ensamblado privado bajo XDG, generar configuración derivada dentro de ese ensamblado y leer `pfx/dosdevices` sin modificarlo.
- **No permitido en 0.5.3:** lanzar Wine/Proton/helper/juego, inyectar DLL, cambiar launch options o escribir en ZZZ/Steam/prefix.
- Antes de seleccionar estrategia: determinar loader identity, relación de prefix, ruta Windows del importer, requisitos runtime/redistribuibles y licencias por componente; probar en un entorno permitido sin bypass/evasión de anti-cheat.
- El siguiente incremento debe ser un experimento de topología/helper no mutante, no un instalador. Apply queda fuera hasta una ADR posterior, un target demostrado y rollback probado.

## Fuentes

- [XXMI Launcher v2.2.1, commit fijado](https://github.com/SpectrumQT/XXMI-Launcher/tree/d56786b8dacb00c35204bff45ff5b8b83bd8962a)
- [Opciones de lanzamiento de XXMI](https://github.com/SpectrumQT/XXMI-Launcher/wiki/More-Start-Options)
- [XXMI Libraries license declaration](https://github.com/SpectrumQT/XXMI-Libs-Package/blob/master/LICENSE.GPL.txt)
- [Topología y evidencia completa de LXMI 0.5.3](../../01_Descubrimiento/topologia_runtime_xxmi_linux.md)
