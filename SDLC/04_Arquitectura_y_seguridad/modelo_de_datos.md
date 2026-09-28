# Modelo de datos inicial

Modelo conceptual para persistencia local; no es un esquema SQLite definitivo.

En LXMI-0.1 a 0.4, los escaneos y planes se ejecutan bajo demanda y sus resultados son transitorios. LXMI 0.5/0.5.1 no incorpora SQLite: persiste paquetes importados y sus manifests/inventarios bajo XDG. LXMI 0.5.3 agrega un ensamblado versionado de runtime en el mismo storage privado. No persiste selecciones Proton, escaneos, assessments, planes de ensamblado/topología ni planes de instalación.

| Tipo de respuesta | Campos principales |
|---|---|
| `SystemInfo` | `os`, `architecture`, `home_directory`, `xdg_data_home`, `xdg_data_dirs` |
| `SteamScanResult` | `status`, `installations`, `issues` |
| `SteamInstallation` | `root_path`, `libraries` |
| `SteamLibrary` | `path`, `is_default` |
| `SteamAppManifest` | `app_id`, `name`, `install_dir` | Datos requeridos parseados de cada manifest válido |
| `SteamGameInstallation` | `installation`, `steam_app_id`, `manifest_name`, `steam_library`, `distribution`, `executable_observation`, `compatdata` | La distribución observada en 0.5.1 es Steam; manifest, carpeta, ejecutable y prefix son estados independientes |
| `GameId` / `IntegrationKind` | `wuthering-waves` / `zenless-zone-zero`; `wwmi` / `zzmi` | Asociación explícita por juego; no inferirla por nombre del paquete ni permitir asociación cruzada |
| `ExecutableObservation` | `expected_names`, `found_path`, `status`, `evidence` | Resultado acotado del filesystem; encontrar el EXE no demuestra que el juego inicie |
| `ProtonCompatData` | `app_id`, `compatdata_path`, `prefix_path`, `status` | Observación del filesystem; no prueba runtime activo ni salud del prefix |
| `CompatibilityTool` | `internal_id`, `display_name`, `path`, `metadata_path`, `source`, `kind`, `version`, `status` | Discovery efímero; Proton, Steam Linux Runtime, otra herramienta o tipo desconocido |
| `GameRuntimePlan` | juego, plataforma, instalación, política, selección, candidates, compatdata, prefix, requisitos, readiness, evidence e issues | Evaluación efímera por juego. Se deriva del resultado actual de discovery; no se guarda en SQLite |
| `ManagedPackageManifest` | package ID, integration/kind, versión raw, source, autenticidad, fecha, archivos y SHA-256 | Manifest JSON y payload importado persisten bajo `XDG_DATA_HOME/lxmi/packages/xxmi/`; el digest detecta cambios, no autentica al upstream |
| `RuntimeAssemblyPlan` / `RuntimeConfigPlan` | runtime ID, App.Root, importer root relativo `ZZMI/`, packages de origen, destinos internos, hashes, target observado, identidad loader y evidencia | DTO transitorio revisable; ensamblar escribe solo bajo el root LXMI, no ejecuta el plan ni modifica paquetes fuente |
| `RuntimeManifest` / `ManagedRuntime` | integración/juego, IDs y versiones fuente, target, loader identity, inventario ensamblado, hashes, config derivada, evidencia y estado de compatibilidad | Persisten bajo `XDG_DATA_HOME/lxmi/runtimes/zenless-zone-zero/zzmi/<runtime-id>/`; el manifest se guarda fuera del payload `ZZMI/`; no significa instalado en juego ni lanzable |
| `DosDevicesReport` / `WindowsPathMapping` | letras drive, symlink y target observados, validez, ruta Linux, resultado y evidencia | Informe efímero de solo lectura; no crea links ni demuestra que Windows runtime pueda abrir/cargar la ruta |
| `LaunchTopologyPlan` | juego, distribución, executable, compatdata/prefix, runtime administrado, Proton candidates/selección, mappings, estrategia, capabilities, requisitos, unknowns y blockers | Efímero y declarativo. `execution_enabled=false`; selección Proton, helper y mismo-prefix requirement pueden continuar `Unknown` |
| `IntegrationAssessment` | juego, integración, distribución, soporte de juego, soporte Steam, compatibilidad Linux/Proton, disponibilidad de paquetes, requisitos y evidencia | Efímero. “Planificable” y “compatible/verificado” son estados distintos |
| `PackageDependency` | paquete requerido, paquete que lo aporta, estado, evidencia | ZZMI requiere XXMI Libraries según metadata/configuración upstream; son paquetes administrados por separado |
| `InstallationPlan` | operaciones propuestas, fuentes, rutas relativas, hashes, backups requeridos, warnings, requisitos y estado de compatibilidad | Efímero, revisable, sin executor. El mapping de 0.5.2 a la carpeta candidata del juego es histórico; no es target autorizado ni activo |

`CompatibilityToolDiscoveryResult` informa estado completo/parcial/no disponible e incidencias de lectura. `GameRuntimePlan` conserva por separado la selección observada y los candidatos disponibles. La selección queda `Unknown` si no hay evidencia fiable específica del juego; no se elige el único candidato por descarte.

El plan también diferencia `NotFound`, `Unknown`, `Unreadable` y `Invalid` para las observaciones de compatdata/prefix. `GameInstallationCompleteness::Unknown` evita asumir que manifest y carpeta presente significan una instalación completa. La evidencia enlaza cada estado con la observación que lo produjo.

`RuntimeReadiness` significa únicamente preparación base observada: no confirma que Steam vaya a usar un runtime específico, que el prefix esté sano, que el juego se ejecute correctamente ni que XXMI esté listo. Runtime discovery no implica runtime selection; runtime selection no implica ejecución.

| Entidad | Campos candidatos | Relaciones / reglas |
|---|---|---|
| `Game` | `id`, `slug`, `display_name`, `runtime_family` | Catálogo de juegos soportados; los slugs no prueban compatibilidad |
| `GameInstallation` | `id`, `game_id`, `root_path`, `executable_path`, `steam_app_id`, `prefix_path`, `detected_at`, `source` | Varias instalaciones por juego; rutas validadas y editables |
| `RuntimeInstallation` | `id`, `game_installation_id`, `runtime_name`, `version`, `root_path`, `config_path`, `status` | Runtime asociado a una instalación; guardar procedencia y versión |
| `Mod` | `id`, `game_id`, `name`, `version`, `source_label`, `archive_sha256`, `original_path`, `managed_path`, `metadata_json` | Sin credenciales ni datos personales; original preservado |
| `ModProfile` | `id`, `game_installation_id`, `name`, `created_at` | Varios perfiles asociados a una instalación |
| `ProfileMod` | `profile_id`, `mod_id`, `enabled` | Restricción que impide asociar un mod a otro juego |
| `OperationJournal` | `id`, `operation`, `source_path`, `target_path`, `backup_path`, `state`, `created_at` | Recuperación ante fallo; no guardar secretos |
| `ConfigBackup` | `id`, `runtime_installation_id`, `path`, `checksum`, `created_at` | Copia con integridad comprobable y restauración explícita |

El estado “activo” debería derivarse del perfil aplicado y del resultado verificado del mecanismo de filesystem, no de un booleano aislado que pueda quedar obsoleto. Revisar si symlinks son compatibles con todos los runtimes objetivo antes de elegirlos.

En 0.5.1, `distribution=Steam` y `steam_app_id=4162040` se sostienen por el manifest local del host. El catálogo de juegos y las integraciones son conceptos distintos: `Zenless Zone Zero` se asocia a `ZZMI`, mientras `Wuthering Waves` se asocia a `WWMI`. HoYoPlay y Manual no tienen scanners en esta versión. `steam_support` y `linux_proton_support` conservan `Unverified`; un candidato Proton no cambia esos estados ni completa `selected_runtime`.

En 0.5.3 el runtime ensamblado por LXMI usa `App.Root/<importer_path>` como topología administrada: `App.Root` está bajo XDG y `importer_path` es `ZZMI/`. Las rutas derivadas del ejecutable de ZZZ solo proporcionan el `target` del `d3dx.ini`; no son destinos de escritura. La copia derivada conserva la identidad upstream `XXMI Launcher.exe`; LXMI nativo no se presenta como ese proceso Windows.
