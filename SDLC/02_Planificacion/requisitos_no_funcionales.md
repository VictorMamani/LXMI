# Requisitos no funcionales iniciales

| ID | Requisito propuesto | Criterio inicial |
|---|---|---|
| RNF-01 | No requerir root para uso cotidiano | Ningún flujo normal eleva privilegios |
| RNF-02 | Proteger archivos fuente de mods | Importar y transformar en copias gestionadas; verificar hash del original en pruebas |
| RNF-03 | Extracción segura | Rechazar traversal, rutas absolutas, enlaces peligrosos y colisiones antes de escribir |
| RNF-04 | Operaciones reversibles | Registrar origen/destino y restaurar configuración previa si una operación falla |
| RNF-05 | Privacidad local | No enviar rutas, logs ni biblioteca a servicios externos sin consentimiento explícito |
| RNF-06 | Errores comprensibles | Mostrar acción afectada, causa conocida y siguiente paso seguro |
| RNF-07 | Observabilidad | Logs locales estructurados con rotación; excluir tokens y datos innecesarios |
| RNF-08 | Compatibilidad explícita | Mostrar sistema operativo, runtime y versiones comprobadas; no inferir soporte general |
| RNF-09 | Mantenibilidad | Mantener separación entre UI Tauri, lógica Rust, filesystem, runtime y fuentes de mods |
| RNF-10 | Accesibilidad | Teclado, foco visible, etiquetas y estados entendibles sin depender solo del color |
| RNF-11 | Rendimiento | Medir primero con biblioteca sintética; fijar umbrales después de tener baseline |
| RNF-12 | Distribución segura | Verificar dependencias, licencias y empaquetado antes de publicar binarios |

## Criterios y evidencia en LXMI-0.1 a 0.4

## Criterios y evidencia incorporados en LXMI-0.5

- **Storage local:** resolver root de LXMI desde XDG existente; no aceptar ruta de destino arbitraria del frontend. Antes de importar se comprueba no-solapamiento con Steam roots/libraries, directorios de juego y compatdata descubiertos.
- **Permisos:** directorios administrados 0700, archivos 0600; import no requiere root. El directorio XDG base debe existir, y el root/ancestros se abren sin seguir symlinks.
- **Integridad:** SHA-256 por archivo, IDs de paquetes por inventario ordenado y comparación completa de payload al listar/planificar. SHA-256 no autentica origen ni firmas.
- **Límites:** 128 MiB por archivo, 512 MiB total, 4096 entradas, 24 niveles. Archive extraction y zip bomb handling no están implementados.
- **Path safety:** validar rutas relativas; rechazar traversal, rutas absolutas, separadores ambiguos, enlaces simbólicos/duros, tipos especiales y colisiones case-insensitive. Staging temporal se promueve sin overwrite; error no aparece como package válido.
- **Plan/reversión:** plan lista acciones/hashes/backups requeridos y advertencias, permanece en datos revisables, `executable=false`. No es transacción ni aprobación para modificar juego. No se probó recuperación ante corte eléctrico.
- **Provenance:** formato declarado del paquete se etiqueta como layout conocido; firma upstream y launch compatibility permanecen `NotAuthenticated` / `NotVerified`.

La importación externa no resuelve races contra procesos hostiles con la misma cuenta ni malware del host. Esta limitación y el threat model se registran en `seguridad_archivos.md`.

- **Solo lectura:** los escáneres/planner inspeccionan filesystem y componen estados; no requieren root, no lanzan Steam/Proton/Wine/juegos y no modifican bibliotecas, manifests, launch options ni prefixes.
- **Errores y estados:** las fallas de manifests/filesystem tienen códigos tipados; compatdata ausente es un estado observable y no un error fatal.
- **Rutas:** alias de Steam se normalizan/deduplican; manifests, `steamapps/common`, directorios de juegos, compatdata, Proton y tools custom con symlinks se rechazan según la política implementada. Las rutas de `install_path` con `..` se rechazan; las rutas absolutas declaradas se aceptan solo si existen como directorios y ninguno de sus componentes es symlink.
- **Lectura acotada:** manifests de compatibility tools y tool manifests limitados a 2 MiB, `version` a 16 KiB y `VERSIONS.txt` a 64 KiB; ningún contenido se ejecuta.
- **Pruebas aisladas:** escenarios Rust usan fixtures y árboles temporales sintéticos. Una comprobación adicional leyó Steam local y evaluó el plan, sin escritura ni ejecución.
- **Persistencia:** el resultado se vuelve a detectar en cada escaneo; SQLite no se incorpora en esta etapa.
- **Límite de evidencia:** el workspace completo y la UI pasan validaciones; la ventana Tauri se abrió y se revisó visualmente. El escaneo/planner real describe una instalación concreta. Los Proton encontrados son candidates; la selección sigue desconocida y readiness no implica lanzamiento, salud del prefix ni compatibilidad XXMI. No se ejecutaron Proton/Wine/juegos.
