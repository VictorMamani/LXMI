# Casos de prueba iniciales

Los casos T-01 a T-01g corresponden a LXMI-0.1; T-20 a T-29 a LXMI-0.2; T-30 en adelante a LXMI-0.3. “Pasó con fixtures” significa que el escenario se ejecutó con rutas sintéticas. El scan real comprueba discovery de filesystem, no selección de Proton, lanzamiento ni presentación visual Tauri.

| ID | Escenario | Resultado esperado | Estado |
|---|---|---|---|
| T-01 | Steam root y biblioteca adicional sintéticas, sin escanear juegos | Mostrar una instalación y las dos bibliotecas válidas | Pasó con fixtures |
| T-01a | No hay Steam en las rutas candidatas | Estado `not_installed`; sin error genérico | Pasó con fixtures |
| T-01b | Raíz Steam existe y no tiene `libraryfolders.vdf` | Estado `configuration_missing` | Pasó con fixtures |
| T-01c | `libraryfolders.vdf` tiene sintaxis inválida | Estado `invalid_configuration` y diagnóstico | Pasó con fixtures |
| T-01d | VDF moderno con `path` anidado y formato legado | Parsear ambas formas de biblioteca | Pasó con fixtures |
| T-01e | `.steam/steam` y `.local/share/Steam` apuntan a la misma instalación | Devolver una raíz canónica, sin duplicados | Pasó con fixtures (Unix) |
| T-01f | Ruta de biblioteca registrada no existe | Omitir esa biblioteca e informar `library_missing` | Pasó con fixtures |
| T-01g | `HOME` ausente para el detector | Estado `home_directory_unavailable`; no panic | Pasó con fixtures |
| T-02 | Ubicación inaccesible o layout no soportado | No fallar silenciosamente; explicar y permitir corrección manual | Pendiente |
| T-03 | Archivo con `../` o ruta absoluta | Rechazar antes de escribir fuera del staging | Fuera de LXMI-0.1 |
| T-04 | Archivo que contiene symlink o hardlink peligroso | Rechazar según política de extracción | Fuera de LXMI-0.1 |
| T-05 | Entradas duplicadas o colisión de nombres | Rechazar o resolver explícitamente sin sobrescritura silenciosa | Fuera de LXMI-0.1 |
| T-06 | Tamaño expandido excede límite | Detener extracción y limpiar staging | Fuera de LXMI-0.1 |
| T-07 | Falla el movimiento final tras importar | Preservar biblioteca anterior y limpiar/recuperar operación incompleta | Fuera de LXMI-0.1 |
| T-08 | Activación con destino fuera del directorio administrado | Rechazar sin seguir ni borrar la ruta externa | Fuera de LXMI-0.1 |
| T-09 | Restaurar backup con checksum distinto | Bloquear restauración automática y presentar diagnóstico | Fuera de LXMI-0.1 |
| T-10 | Perfil contiene un mod asociado a otro juego | Rechazar asociación | Fuera de LXMI-0.1 |
| T-11 | Transformación de configuración | Hash del original permanece igual; salida reproducible en carpeta gestionada | Fuera de LXMI-0.1 |
| T-12 | Error durante aplicación de perfil | Revertir al estado previo y conservar journal del resultado | Fuera de LXMI-0.1 |

Los límites de tamaño, tipos de entrada y comportamiento ante symlinks se definirán con una implementación concreta y dependencias revisadas.

## LXMI-0.2 — manifests, juegos y compatdata

Todos estos escenarios usan fixtures sintéticos y se ejecutan dentro de los tests de los crates Rust. No prueban Steam real ni la ventana nativa Tauri.

| ID | Escenario | Resultado esperado | Estado |
|---|---|---|---|
| T-20 | Manifest ACF válido con tabs, saltos de línea y campos adicionales | Leer AppID, nombre e installdir; ignorar campos desconocidos | Pasó con fixtures |
| T-21 | AppID ausente/inválido, campos requeridos ausentes, clave duplicada o ACF truncado | Error tipado; no panic | Pasó con fixtures |
| T-22 | `installdir` absoluto, anidado, con `..` o separador Windows | Rechazar antes de inspeccionar ruta externa | Pasó con fixtures |
| T-23 | AppID de archivo y payload diferentes | Omitir manifest y emitir aviso | Pasó con fixtures |
| T-24 | Biblioteca Steam sin manifests | Scan completo, sin juegos reconocidos | Pasó con fixtures |
| T-25 | Varias bibliotecas con app desconocida y Wuthering Waves | Contar manifests válidos, ignorar app desconocida e identificar Wuthering Waves por AppID | Pasó con fixtures |
| T-26 | Manifest inválido junto a uno válido; carpeta del juego ausente | Continuar el scan y conservar el juego con estado/aviso de carpeta ausente | Pasó con fixtures |
| T-27 | AppID de Wuthering Waves sin manifest en bibliotecas | No reportar juego; resultado “no encontrado” si el scan está completo | Pasó con fixtures |
| T-28 | `compatdata` ausente, presente sin `pfx`, y presente con `pfx` | Devolver respectivamente `NotFound`, `CompatDataFound`, `PrefixFound` | Pasó con fixtures |
| T-29 | `compatdata`, `pfx`, `steamapps/common` o manifest como symlink/no directorio | Marcar ruta inválida y no seguir symlink | Pasó con fixtures |

## LXMI-0.3 — Proton Runtime Discovery

| ID | Escenario | Resultado esperado | Estado |
|---|---|---|---|
| T-30 | Tool manifest layer Proton y entrypoint regular, con folder name arbitrario | Identificar Proton válido por estructura, no por nombre | Pasó con fixtures |
| T-31 | Custom compatibilitytool.vdf válido y directorio de instalación resuelto | Identificar metadata custom, ID, display name, path y source | Pasó con fixtures; no se encontró custom tool en Steam local |
| T-32 | Steam Linux Runtime con marker de layer y `VERSIONS.txt` | Clasificar como Steam Linux Runtime, no Proton; leer depot version disponible | Pasó con fixtures y Steam local |
| T-33 | Carpeta llamada Proton sin estructura reconocible | No clasificarla como Proton | Pasó con fixtures |
| T-34 | Runtime Proton con `version` ausente | Mantener Proton válido y mostrar versión desconocida | Pasó con fixtures |
| T-35 | Marker Proton sin entrypoint, VDF inválido o tool incompleta | Reportar candidato incompleto/inválido sin abortar el resto | Pasó con fixtures |
| T-36 | `install_path` con `..` o componentes symlink | Rechazar y no seguir el path | Pasó con fixtures |
| T-37 | Scan con Steam local | Encontrar Proton Experimental y tres SLR; reportar el symlink `Steam.dll` sin seguirlo | Pasó en lectura local |
| T-38 | Wuthering Waves detectado con compatdata ausente y runtimes disponibles | Mostrar compatdata ausente; no atribuirle Proton seleccionado | Confirmado por scan local y ventana Tauri |

En LXMI-0.3 pasaron 57 tests del workspace Rust, `cargo check`, Clippy con `-D warnings`, `cargo fmt --check` y validaciones frontend. La ventana Tauri se inició y revisó visualmente con los resultados del scan local.
