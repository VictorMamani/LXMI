# LXMI Loader Lab (desarrollo)

Estas herramientas prueban el export `Inject` del `3dmloader.dll` oficial contra un único proceso Windows creado por LXMI. No aceptan PID, nombre de proceso ni ruta de DLL arbitrarios. El runner obtiene todos los paths desde su propia ubicación bajo `lxmi/tests/loader-v1/`.

El target es `lxmi-loader-test-target.exe`; la DLL inocua es `lxmi-loader-test.dll`. La DLL solo escribe un marker JSON dentro del directorio de resultados del laboratorio cuando el nombre de su propio proceso coincide exactamente con el target LXMI.

`3dmloader.dll` no se copia al repositorio. El backend lo obtiene de un paquete XXMI Libraries que el store verifica como release oficial y copia al directorio privado del laboratorio.

## Build

Requisitos en Ubuntu: `gcc-mingw-w64-x86-64` y `mingw-w64-x86-64-dev`. No se instala nada automáticamente.

Desde `SDLC/05_Construccion/aplicacion/`:

```bash
bash tools/lxmi-loader-lab/build.sh
```

El resultado queda en `target/x86_64-pc-windows-gnu/release/lxmi-loader-lab/`. El build usa el compilador MinGW del host. Los archivos se validan y se copian al storage XDG de LXMI al pulsar **Prepare Loader Lab** en la UI.

## Restricciones

- `direct_inject` llama al export upstream con el PID del `CreateProcessW` de su hijo, no busca procesos.
- `hook` no se prueba: upstream instala un hook global de ventanas.
- El runner solo inicia el ejecutable fijo del test; no hay argumento de ruta.
- La DLL verifica el nombre de su proceso y un nonce de 128 bits.
- No usar con ZZZ, Steam, HoYoPlay ni otro proceso.
- Esta carpeta contiene únicamente código de test LXMI; no contiene código fuente upstream ni redistribuye binarios upstream.
