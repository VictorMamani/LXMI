# Modelo de datos inicial

Modelo conceptual para persistencia local; no es un esquema SQLite definitivo ni se ha implementado.

LXMI-0.1/0.2 no persiste datos. Los escaneos se ejecutan bajo demanda; sus respuestas transitorias incluyen estos conceptos:

| Tipo de respuesta | Campos principales |
|---|---|
| `SystemInfo` | `os`, `architecture`, `home_directory`, `xdg_data_home`, `xdg_data_dirs` |
| `SteamScanResult` | `status`, `installations`, `issues` |
| `SteamInstallation` | `root_path`, `libraries` |
| `SteamLibrary` | `path`, `is_default` |
| `SteamAppManifest` | `app_id`, `name`, `install_dir` | Datos requeridos parseados de cada manifest válido |
| `SteamGameInstallation` | `installation`, `steam_app_id`, `manifest_name`, `steam_library`, `compatdata` | El flujo muestra por ahora solo juegos del registro soportado |
| `ProtonCompatData` | `app_id`, `compatdata_path`, `prefix_path`, `status` | Observación del filesystem; no prueba runtime activo ni salud del prefix |

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
