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

## Criterios y evidencia en LXMI-0.1/0.2/0.3

- **Solo lectura:** los escáneres inspeccionan filesystem; no requieren root, no lanzan Steam/juegos y no modifican bibliotecas, manifests ni prefixes.
- **Errores y estados:** las fallas de manifests/filesystem tienen códigos tipados; compatdata ausente es un estado observable y no un error fatal.
- **Rutas:** alias de Steam se normalizan/deduplican; manifests, `steamapps/common`, directorios de juegos, compatdata, Proton y tools custom con symlinks se rechazan según la política implementada. Las rutas de `install_path` con `..` se rechazan; las rutas absolutas declaradas se aceptan solo si existen como directorios y ninguno de sus componentes es symlink.
- **Lectura acotada:** manifests de compatibility tools y tool manifests limitados a 2 MiB, `version` a 16 KiB y `VERSIONS.txt` a 64 KiB; ningún contenido se ejecuta.
- **Pruebas aisladas:** escenarios Rust usan fixtures y árboles temporales sintéticos. Una comprobación adicional leyó Steam local, sin escritura ni ejecución.
- **Persistencia:** el resultado se vuelve a detectar en cada escaneo; SQLite no se incorpora en esta etapa.
- **Límite de evidencia:** el workspace completo y la UI web compilan/pasan validaciones; la ventana Tauri se abrió y se revisó visualmente. El escaneo real es evidencia de discovery de una instalación concreta. No se ejecutaron Proton/Wine/juegos ni se verificó compatibilidad.
