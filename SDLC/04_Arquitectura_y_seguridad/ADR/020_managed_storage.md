# ADR-020 — Almacenamiento administrado XXMI

Estado: adoptado en 0.5; Ubuntu/Linux únicamente.

## Decisión

Usar `SystemInfo.xdg_data_home` existente para resolver `lxmi`, con fallback XDG ya implementado en core. El directorio XDG base debe existir; import no crea ancestros externos. No se acepta un root de almacenamiento desde IPC. Se comprueba solapamiento con Steam/bibliotecas/juegos descubiertos antes de importar.

| Ruta bajo LXMI | Contenido | Limpieza |
|---|---|---|
| `packages/xxmi/<SHA-256>/payload/` | Copia de archivos importados, sin transformación | Estado administrado; no borrar mientras se referencie. No hay botón de borrado en 0.5. |
| `packages/xxmi/<SHA-256>/lxmi-package.json` | Inventario, origen declarado, versión raw, fecha Unix, referencia de layout, hashes | Necesario para verificar el paquete. No va dentro del payload upstream. |
| `staging/import-<pid>-<counter>/` | Import en curso | Limpieza automática al fallar; tras caída se puede revisar y eliminar manualmente cuando LXMI esté cerrado. Nunca se lista como paquete válido. |
| `runtimes/wwmi/` | Destino propuesto de despliegue | **No creado por 0.5**. No es cache. |
| `cache/`, `logs/`, `metadata/` | Reservados si aparece una necesidad | No se crean carpetas vacías por anticipado. Logs actuales salen por tracing. |

Staging y packages comparten filesystem. Promoción por `renameat2(RENAME_NOREPLACE)` evita overwrite; las operaciones con contenido duplicado revalidan el paquete existente y lo reutilizan. Archivos 0600 y directorios 0700, sin conservar bits de ejecución. Originales inmutados. Los directorios vacíos upstream no forman parte del fingerprint; el inventario representa archivos, no una imagen del filesystem.

## Seguridad

Acceso anclado a descriptores de directorio; cada componente externo se abre con O_NOFOLLOW. Los paths `/proc/self/fd` los construye LXMI desde FDs abiertos; no son una excepción general para enlaces del paquete. Se rechazan symlinks, hardlinks, archivos especiales y colisiones por mayúsculas. La promoción no reemplaza otro directorio.

SHA-256 detecta diferencias respecto al import. No autentica el origen ni defiende contra un atacante con acceso de escritura a toda la cuenta que reescriba payload e inventario conjuntamente. No se promete resistencia a pérdida de energía: se sincronizan archivos y puntos de promoción, y se revalida al listar o planificar. No se realizó fault injection de cortes de energía.

Dependencias directas añadidas, ya presentes transitivamente en Cargo.lock: `sha2` para hashing, `serde/serde_json` para metadata/IPC y `libc` para flags Linux y rename no-replace. No se añadió un framework, base de datos ni parser de archives.
