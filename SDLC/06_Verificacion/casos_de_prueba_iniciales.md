# Casos de prueba iniciales

Los casos T-01 a T-01g corresponden a LXMI-0.1; T-20 a T-29 a LXMI-0.2; T-30 a T-38 a LXMI-0.3; T-40 en adelante a LXMI-0.4 y siguientes. “Pasó con fixtures” significa que el escenario se ejecutó con rutas sintéticas. El scan real comprueba discovery y la evaluación del planner con el estado local; no prueba qué runtime elegirá Steam ni permite lanzar el juego.

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

## LXMI-0.4 — Runtime Planning & Launch Readiness

| ID | Escenario | Resultado esperado | Estado |
|---|---|---|---|
| T-40 | Wuthering Waves presente, un candidato Proton válido, sin compatdata | `NeedsInitialization`; candidate disponible, selección desconocida | Pasó con fixtures |
| T-41 | Juego presente y varios candidatos Proton | Listar todos los candidatos; no elegir uno automáticamente | Pasó con fixtures |
| T-42 | Un único candidato Proton | Mantener selección `Unknown` sin evidencia por juego | Pasó con fixtures (`single_proton_candidate_does_not_imply_selected_runtime`) |
| T-43 | Juego presente, `compatdata/pfx` observado y selección desconocida | Prefix queda como candidato; no inferir runtime ni readiness completa | Pasó con fixtures |
| T-44 | Discovery completo sin manifest del juego | `Blocked` y game `NotFound` | Pasó con fixtures |
| T-45 | Discovery de tools completo sin Proton válido | `Incomplete`; no promover otra tool | Pasó con fixtures |
| T-46 | Solo compatibility tools desconocidas | Cero candidatos Proton | Pasó con fixtures |
| T-47 | Steam Linux Runtime disponible | No clasificar SLR como Proton | Pasó con fixtures |
| T-48 | Compatdata/prefix ilegible por permisos | Preservar `Unreadable`, issue y readiness `Unknown` | Pasó con fixtures |
| T-49 | Discovery parcial de juegos sin Wuthering Waves | Estado `Unknown`, no afirmar que no está instalado | Pasó con fixtures |
| T-50 | Scan local Steam y planner desde la aplicación Tauri | Wuthering Waves presente; compatdata/pfx ausentes; dos candidatos Proton, tres SLR; selección desconocida; `NeedsInitialization`; scan completo | Comprobado en este equipo, solo lectura |

En LXMI-0.4 pasaron 69 tests del workspace Rust, `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy con `-D warnings` y las validaciones frontend. El scan real no ejecutó Steam, Proton, Wine ni el juego.

## LXMI-0.5 — XXMI / WWMI Integration Foundation

Los escenarios T-60 a T-104 son 45 tests del crate `lxmi-xxmi`. Salvo validación del corpus oficial externa a la suite, todos usan árboles/file payloads de **FIXTURE** sintéticos y no contienen runtime funcional.

| Grupo | Casos | Resultado |
|---|---|---|
| Package/metadata | WWMIv1 válida; recursos/config faltantes; namespace desconocido; parser raw version; XXMI Libraries separado; DLL aislada insuficiente; manifest/signatures ausentes o inválidos | Pasó en fixtures |
| Integrity | SHA-256 conocido; payload modificado; manifest LXMI alterado; verificación obligatoria al listar/planificar; IDs duplicados se reusan sin overwrite | Pasó en fixtures |
| Path/source safety | Traversal, absolute, separadores Windows, dot/empty components, nombres case collision, symlink de archivo/ancestro/root/destino, hardlink, storage/source solapado | Pasó en fixtures |
| Resource bounds | Máximo archivo, total, entries, profundidad; `.zip` explícitamente rechazado | Pasó en fixtures |
| Storage/staging | Root XDG, no crear en list/discovery/plan; modos privados; promoción y manifest lateral; import incompleto no listado; fuente sin cambios | Pasó en fixtures |
| Runtime discovery | Absent normal, estructura presente con compatibilidad `NotVerified`, incomplete e unreadable | Pasó en fixtures |
| Assessment/plan | WWMI con juego equivocado rechazado; paquetes/runtime no implican compatibilidad; selección queda de planner; juego ausente; create/replace + previous hash/backup; plan no modifica snapshots externos | Pasó en fixtures |
| No execution | Contenido script ejecutable sintético no se invoca durante inspección/import | Pasó en fixtures |
| IPC contract | `PackageKind` serializa WWMI y XXMI Libraries en la forma esperada por React | Pasó con test Rust |
| IPC contract | `PackageKind` serializa WWMI/XXMI Libraries en la forma que consume React | Pasó en test Rust |
| Corpus upstream | Releases oficiales v1.0.0 WWMI y v1.1.7 Libraries validadas/importadas mediante `local_review` en storage `/tmp`; no se ejecutan DLL ni scripts | Comprobado fuera de test runner; ver `verificacion_0_5.md` |
| Game local anterior | Directorio presente, ejecutable candidato no encontrado, compatdata/pfx absent; runtimes no inferidos como selección | Comprobado en filesystem local de solo lectura (2026-09-27) |
| Game local actual | Sin manifest Wuthering Waves; carpetas Proton sin metadata/entrypoint suficiente; compatdata absent; readiness `Blocked` | Comprobado en filesystem local de solo lectura (2026-09-28) |

Total workspace 0.5: 114 tests pasan (69 previos + 45 nuevos). La interacción del botón Import/Review a través de IPC en la ventana Tauri queda **NO COMPROBADA**; Tauri sí compiló e inició.

## LXMI-0.5.1 — Zenless Zone Zero / ZZMI

Los tests de paquetes y planes usan **FIXTURE**; solo el discovery Steam/prefix tiene un caso de host deliberadamente `#[ignore]`. Este no forma parte de `cargo test --workspace` normal y debe ejecutarse manualmente en un host donde ZZZ siga presente. El contrato upstream se contrastó el 2026-09-28, pero LXMI no descargó ni importó la release ZZMI.

| ID | Escenario | Resultado esperado | Estado |
|---|---|---|---|
| T-105 | Manifest Steam de ZZZ con AppID `4162040` | Registrar ZZZ y derivar la instalación desde `installdir` | Pasó con fixture; confirmado contra manifest local |
| T-106 | Instalación con `ZenlessZoneZero.exe` dentro de `games/ZenlessZoneZero Game/` | Encontrar el ejecutable en el scan acotado, sin seguir symlinks | Pasó con fixture y scan local |
| T-107 | Manifest ZZZ con carpeta o ejecutable ausente | Conservar estados separados y no declarar la instalación lista | Pasó con fixture |
| T-108 | ZZMI con anclas `d3dx.ini`, `Core/ZZMI/main.ini` y includes estructurales | Reconocer contrato mínimo y versión raw sin ejecutar contenido | Pasó con fixture; contrato contrastado con upstream |
| T-109 | ZZMI incompleto o versión raw ausente | Reportar incompleto/desconocido; no inferir versión | Pasó con fixture |
| T-110 | ZZMI asociado a Wuthering Waves o WWMI asociado a ZZZ | Rechazar asociación cruzada | Pasó con fixture; regresión WWMI conservada |
| T-111 | ZZMI presente, XXMI Libraries ausente | Mantener dependencia insatisfecha; no clasificar conjunto como autosuficiente | Pasó con fixture |
| T-112 | ZZMI y XXMI Libraries presentes en storage administrado | Permitir construir un plan descriptivo, con compatibilidad de plataforma no verificada | Pasó con fixture |
| T-113 | Un único candidato Proton local para ZZZ | Selección de Proton permanece `unknown`; readiness no se promueve a Ready | Pasó con fixture y snapshot host-only |
| T-114 | Plan ZZMI generado | Destino solo administrado por LXMI; no escribe ZZZ, Steam, compatdata ni prefix | Pasó con fixture mediante comparación de estado |
| T-115 | Host local con manifest ZZZ, EXE y `compatdata/4162040/pfx` | Mostrar Steam, AppID, EXE y prefix candidato; no seleccionar Proton | Comprobado por test host-only `#[ignore]` y filesystem read-only |
| T-116 | Steam/Linux/Proton y ejecución ZZMI real | No declarar compatibilidad sin prueba específica | `NO COMPROBADO`; issue upstream de Steam/Linux seguía abierto |

El test host-only se ejecutó explícitamente con `cargo test -p lxmi-desktop local_steam_scan_finds_zzz_without_inferring_proton_selection -- --ignored --nocapture`. No se activó desde la ventana Tauri y no se comparó hash/snapshot completo del árbol real antes/después; las invariantes de no escritura se comprueban con fixtures y la ruta real ejercitada es read-only.
