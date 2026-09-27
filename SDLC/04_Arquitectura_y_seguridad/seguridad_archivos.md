# Seguridad de archivos y límites de integración

## LXMI-0.1/0.2: detección de solo lectura

- Los incrementos solo consultan metadata y leen `libraryfolders.vdf`, `appmanifest_*.acf` y rutas candidatas de `compatdata`; no modifican Steam, juegos, manifests ni prefixes.
- Rutas candidatas se limitan a directorios conocidos bajo home/XDG; su procedencia queda visible en resultados/diagnósticos.
- Resolver un alias simbólico de Steam sirve para canonizar y deduplicar la detección; no se escribe ni se elimina contenido siguiendo ese enlace.
- El parser KeyValues procesa texto local, limita cada archivo a 2 MiB y la profundidad anidada a 64 niveles, y devuelve errores con offset; no interpreta contenido como comando.
- `installdir` debe ser un nombre de directorio relativo simple. El scanner rechaza manifests con rutas absolutas, separadores, `..`, AppID inválido o discrepancia entre AppID y nombre del archivo.
- El scanner no sigue symlinks en manifests, `steamapps/common`, rutas de juego, `compatdata` ni `pfx`. Un `pfx` presente se informa como candidato, no como prefix sano o Proton activo.
- Los tests usan directorios temporales, no recorren ni modifican bibliotecas reales.

## Archivos de mods

- Tratar archivos comprimidos como entrada no confiable.
- Inspeccionar nombres de entradas antes de extraer: rechazar rutas absolutas, `..`, separadores ambiguos, enlaces simbólicos/duros no autorizados, nombres duplicados y colisiones.
- Canonicalizar el directorio de destino y demostrar que cada ruta final permanece dentro de él.
- Limitar cantidad de entradas y tamaño descomprimido para evitar agotamiento de disco/memoria.
- Extraer primero a directorio temporal nuevo; validar antes de mover atómicamente a la biblioteca.
- No seguir enlaces al activar/desactivar ni borrar recursivamente una ruta que salga del área de LXMI.
- Nunca editar el original. Guardar hash y ubicación de fuente; escribir transformaciones en área gestionada.
- Antes de reemplazar configuración del runtime, crear backup y comprobar que se puede restaurar.

## Procesos y permisos

- No requerir root para flujos normales.
- No concatenar argumentos del usuario en una shell. Usar APIs de proceso con ejecutable y argumentos separados.
- Mostrar y registrar claramente qué ejecutable, prefix, argumentos y variables se usarán.
- Validar rutas permitidas y evitar ejecutar binarios incluidos en un archivo de mod.
- Redactar rutas sensibles y evitar registrar tokens, cookies o variables secretas.

## Runtime, juegos y distribución

Antes de integrar o distribuir, revisar repositorios oficiales, licencias, avisos de copyright y políticas aplicables del juego/runtime. No modificar anti-cheat, DRM ni controles, no simular compatibilidad y no automatizar acciones que puedan poner una cuenta en riesgo. Si los límites no están claros, mantener pruebas en fixtures o juegos de laboratorio autorizados.

La licencia de LXMI no debe elegirse hasta conocer dependencias y componentes que se empaquetarían. Este documento no es una conclusión legal.
