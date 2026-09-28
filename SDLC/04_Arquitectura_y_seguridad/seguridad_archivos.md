# Seguridad de archivos y límites de integración

## LXMI-0.1 a 0.4: detección y planificación de solo lectura

- Los incrementos solo consultan metadata y leen `libraryfolders.vdf`, `appmanifest_*.acf`, metadata de compatibility tools y rutas candidatas de `compatdata`; no modifican Steam, juegos, Proton ni prefixes.
- Rutas candidatas se limitan a directorios conocidos bajo home/XDG; su procedencia queda visible en resultados/diagnósticos.
- Resolver un alias simbólico de Steam sirve para canonizar y deduplicar la detección; no se escribe ni se elimina contenido siguiendo ese enlace.
- El parser KeyValues procesa texto local, limita cada archivo a 2 MiB y la profundidad anidada a 64 niveles, y devuelve errores con offset; no interpreta contenido como comando.
- `installdir` debe ser un nombre de directorio relativo simple. El scanner rechaza manifests con rutas absolutas, separadores, `..`, AppID inválido o discrepancia entre AppID y nombre del archivo.
- El scanner no sigue symlinks en manifests, `steamapps/common`, rutas de juego, `compatdata` ni `pfx`. Un `pfx` presente se informa como candidato, no como prefix sano o Proton activo.
- Proton discovery solo lee `toolmanifest.vdf`, `compatibilitytool.vdf`, `proton` como metadata de archivo, `version` y `VERSIONS.txt`; no ejecuta entrypoints ni sigue symlinks.
- `install_path` custom no acepta `..`, componentes de prefijo ni rutas relativas con root. Los paths absolutos admitidos por el formato se validan como directorios reales, recorriendo componentes con `symlink_metadata` y rechazando enlaces. No se escribe en esos paths.
- Un nombre de carpeta no es prueba suficiente de runtime. La clasificación usa layer metadata y archivos estructurales; los Steam Linux Runtime se mantienen separados de Proton.
- `lxmi-runtime` combina resultados ya detectados para formar un plan efímero. No abre rutas ni ejecuta procesos; runtime discovery no implica selection y selection no implica launching.
- Los tests usan directorios temporales, no recorren ni modifican bibliotecas reales. El scan de integración local es independiente y solo lectura.

## Archivos de mods — trabajo futuro

LXMI 0.5 aún no implementa una biblioteca de mods ni activación/desactivación. Las reglas de gestión de mods que siguen son criterios para incrementos futuros.

## LXMI-0.5 — paquetes XXMI y WWMI

- Las fuentes locales son entrada no confiable. LXMI importa directorios, no ZIP/7z/RAR; la extracción externa no la proporciona el importador.
- Contrato actual acotado a WWMI `WWMIv1` comprobado en el release WWMI v1.0.0, ZZMI por anclas estructurales verificadas en upstream y XXMI Libraries como paquete separado. El contrato no es un inventario exhaustivo, intérprete INI, verificación de firma ni prueba de ejecución.
- La inspección es pasiva, finita y read-only sobre candidatos explícitos. No busca recursivamente todo el disco, no ejecuta DLL/EXE/script y no escribe en game/prefix/Steam.
- SHA-256 cubre ruta/tamaño/digest de cada archivo y se revalida antes de listar/planificar. Detecta cambios accidentales posteriores; no autentica el publisher. `Manifest.json.signatures` se conserva como declaración upstream y no se verifica con la clave upstream.
- Límites por defecto: 4096 entradas, profundidad 24, 128 MiB por archivo y 512 MiB total. No se implementa ZIP bomb mitigation todavía porque no se abre ningún archive. Los límites son del importador de directorios; revalidación del managed payload aplica los mismos límites.
- Se rechazan rutas absolutas, traversal, separadores ambiguos, nombres de control, colisiones case-insensitive, symlink, hardlink y entradas no regulares. Almacén raíz y ancestros se abren sin seguir symlink; salida se ancla a directorios LXMI privados y promoción no reemplaza IDs existentes.
- Fuente y almacenamiento no pueden solaparse. El servicio Tauri comprueba separación con Steam, libraries, game y compatdata observados antes de import. Cuando Steam discovery no da límites confiables, se bloquea import. Root se resuelve solo desde `SystemInfo.xdg_data_home`; no viene del frontend.
- Staging incompleto nunca se cuenta como package válido. Error limpia su temp; tras crash puede persistir temporal desconocido que necesita inspección manual antes de limpieza. `packages/xxmi/<digest>` solo se considera verificado tras volver a calcular inventory y comparar manifest.
- Un usuario o malware que corre con los mismos permisos de la cuenta puede alterar su propio storage. 0.5 no pretende ser sandbox contra procesos de la misma cuenta.
- La promoción usa rename no-replace y syncs básicos, pero no es una operación transaccional resistente a corte eléctrico demostrada. 0.6 requiere journal durable, backup/rollback ensayado y recuperación crash.
- Packages ausentes, runtimes ausentes, no compatibles aún y versiones desconocidas son observaciones/estados; no son errores fatales. Invalid metadata, permiso denegado, checksum mismatch y unsafe path sí son diagnósticos diferenciados.

Threats incluidos: malicious path names, symlink/hardlink escape, overwrite, duplicate/case collision, corrupt bytes, huge directory, stale metadata. No resueltos aquí: filesystems hostiles remotos, agotamiento por millones de directorios sobre límite (se detiene al exceder), crash/power loss, firma/origen, races de otro proceso con la misma cuenta, malware dentro del contenido en una eventual integración runtime.

La revisión upstream identificó declaraciones GPLv3, MIT/BSD/PCRE2 y contenidos que pueden tener otros titulares/licencias. No distribuir paquetes/recursos de terceros desde LXMI hasta cerrar matriz y obligaciones. Revisar `01_Descubrimiento/ecosistema_xxmi_wwmi.md` y ADR-021.

## LXMI-0.5.1 — ZZZ y ZZMI

- El scanner identifica Zenless Zone Zero por el AppID del manifest Steam `4162040`, deriva el directorio desde `installdir` y busca únicamente los nombres de ejecutable confirmados en el registro. La inspección no ejecuta el EXE, no sigue symlinks ni implica que la instalación esté completa.
- `ZZMI` solo se asocia a `ZenlessZoneZero`; `WWMI` solo a `Wuthering Waves`. Rechazar una asociación cruzada es una regla de dominio, no una medida de compatibilidad de plataforma.
- La validación ZZMI inspecciona una lista estructural mínima contrastada con el árbol upstream. Se trata como entrada local no confiable: no interpreta INI, no ejecuta scripts/binarios y no verifica la autenticidad de la release.
- `XXMI Libraries` se representa como una dependencia distinta. Una carpeta ZZMI estructuralmente válida no demuestra que las bibliotecas requeridas estén disponibles ni que la combinación pueda ejecutarse.
- El assessment mantiene separados `game_support`, distribución detectada, soporte de Steam, compatibilidad Linux/Proton, selección de Proton y estado del prefix. Steam/Linux/Proton permanecen `Unverified` en ausencia de evidencia de ejecución autorizada.
- El plan solo describe operaciones propuestas hacia el almacenamiento administrado por LXMI. No contiene apply ni destinos dentro de ZZZ, Steam o `compatdata`.

**Límite de autenticidad:** los hashes SHA-256 del almacenamiento LXMI comprueban que el contenido coincide con el inventario creado al importar; no validan la firma upstream de ZZMI. La metadata de firma que usa XXMI Launcher no ha sido implementada en LXMI.

## Controles requeridos antes de archives y activación de mods

Estos controles no implican soporte actual de archivos comprimidos ni instalación de mods. En 0.5/0.5.1 se importan únicamente directorios locales al almacenamiento LXMI descrito arriba.

- Tratar cualquier archive futuro como entrada no confiable.
- Inspeccionar nombres de entradas antes de extraer: rechazar rutas absolutas, `..`, separadores ambiguos, enlaces simbólicos/duros no autorizados, nombres duplicados y colisiones.
- Anclar el destino a un descriptor/directorio controlado y demostrar que cada ruta permanece dentro de él.
- Limitar cantidad de entradas y tamaño descomprimido para evitar agotamiento de disco/memoria.
- Extraer primero a directorio temporal nuevo; validar antes de promoverlo a la biblioteca.
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

## LXMI-0.5.2 — releases y ZIP

- La consulta/descarga requiere acción explícita. El provider admite solo `leotorrez/ZZMI-Package` y `SpectrumQT/XXMI-Libs-Package`; pruebas unitarias usan provider fixture offline.
- Metadata, descarga y package administrado se separan. La provenance guarda repositorio, ID/tag/commit, URL/asset, fechas de publicación/consulta/descarga, firma, digest esperado/observado y companion asset.
- Solo HTTPS hacia API GitHub y hosts de assets permitidos. Redirects se siguen manualmente con máximo 5 y se revalida cada destino. Se bloquean esquema local, IP literal, userinfo, puerto no estándar y host no allowlisted. Metadata se limita a 1 MiB; los ZIP a 32 MiB y companion JSON a 64 KiB; timeout de conexión 10 s y total 30 s.
- El stream se guarda como `.partial` en `XDG_CACHE_HOME/lxmi/downloads` (fallback `~/.cache/lxmi/downloads`), con límite real de bytes y SHA-256. Solo se promueve cuando termina, coincide el tamaño del API y, si GitHub publica digest, también coincide ese hash.
- Firma de release se verifica sobre bytes exactos con las claves upstream fijadas y ECDSA P-384/SHA-256. Firma ausente/inválida/algoritmo no soportado bloquea la importación oficial. Para XXMI Libraries, `Manifest.json` separado se coteja con el digest GitHub y sus firmas de `3dmloader.dll`, `d3d11.dll` y `d3dcompiler_47.dll` se verifican criptográficamente. El documento Manifest no tiene firma propia; no describirlo como firmado.
- ZIP se extrae con límites a staging privado: 4096 entries, profundidad 24, 128 MiB por archivo, 512 MiB descomprimidos, compresión hasta 1000:1. Rechaza traversal, absolute/Windows paths, separadores ambiguos, colisiones case-insensitive, enlaces y tipos especiales; stream valida tamaños y CRC. RAR/7z no están soportados.
- Solo staging ya validado puede promoverse a `$XDG_DATA_HOME/lxmi/packages/xxmi/<digest>` con no-replace. La operación de release no escribe al juego, Steam, compatdata o prefix ni ejecuta contenido.
- SHA-256 es integridad; la firma upstream vincula el byte sequence con la clave fijada; ninguna indica que el archivo sea seguro al ejecutar, compatible o distribuible. El store vuelve a comprobar su inventario local; no vuelve a verificar la firma del ZIP al listar si el ZIP original no se conserva en el managed package.
- Riesgo residual: límites altos pueden consumir espacio; descompresor/OS y procesos del mismo usuario quedan fuera de un sandbox; DNS/TLS se confían dentro de la allowlist; licencias por componente y ZZZ Steam/Linux/Proton siguen sin resolver. Apply permanece ausente.
