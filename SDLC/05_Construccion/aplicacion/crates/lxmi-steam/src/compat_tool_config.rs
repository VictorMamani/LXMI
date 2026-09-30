//! Read-only inspection of Steam's text `config/config.vdf` compatibility mappings.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::vdf::{parse_key_values_document, KeyValuesValue, VdfParseError};

const MAX_CONFIG_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamCompatToolConfigStatus {
    Read,
    Missing,
    Invalid,
    Unreadable,
    UnsafePath,
    TooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamCompatToolMapping {
    /// `0` denotes the global mapping; otherwise this is the mapped Steam AppID.
    pub app_id: u32,
    /// Raw internal tool name from Steam config. An empty string is preserved as evidence.
    pub internal_name: Option<String>,
    pub config: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SteamCompatToolMappings {
    pub per_app: Option<SteamCompatToolMapping>,
    pub global_default: Option<SteamCompatToolMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamCompatToolConfigInspection {
    pub status: SteamCompatToolConfigStatus,
    pub config_path: PathBuf,
    pub mappings: Option<SteamCompatToolMappings>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamCompatToolConfigParseError {
    Document(VdfParseError),
    SectionMissing(&'static str),
    DuplicateKey(String),
    ExpectedObject(&'static str),
    ExpectedScalar(&'static str),
}

impl std::fmt::Display for SteamCompatToolConfigParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document(error) => write!(formatter, "invalid KeyValues document: {error}"),
            Self::SectionMissing(section) => write!(formatter, "{section} section is missing"),
            Self::DuplicateKey(key) => write!(formatter, "{key} appears more than once"),
            Self::ExpectedObject(key) => write!(formatter, "{key} must be an object"),
            Self::ExpectedScalar(key) => write!(formatter, "{key} must be a scalar value"),
        }
    }
}

impl std::error::Error for SteamCompatToolConfigParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Document(error) => Some(error),
            _ => None,
        }
    }
}

/// Parses the per-AppID entry and Steam's special `0` global mapping.
/// A global mapping is reported separately and is never promoted to a per-game selection.
pub fn parse_steam_compat_tool_mappings(
    input: &str,
    app_id: u32,
) -> Result<SteamCompatToolMappings, SteamCompatToolConfigParseError> {
    let document =
        parse_key_values_document(input).map_err(SteamCompatToolConfigParseError::Document)?;
    let install_config = required_object(&document, "InstallConfigStore")?;
    let software = required_object(install_config, "Software")?;
    let valve = required_object(software, "Valve")?;
    let steam = required_object(valve, "Steam")?;
    let mappings = required_object(steam, "CompatToolMapping")?;

    Ok(SteamCompatToolMappings {
        per_app: parse_mapping(mappings, &app_id.to_string(), app_id)?,
        global_default: parse_mapping(mappings, "0", 0)?,
    })
}

/// Inspects `<steam-root>/config/config.vdf` without following a symlink at the config
/// directory or file. A missing file or mapping is evidence absence, not a runtime choice.
pub fn inspect_steam_compat_tool_config(
    steam_root: &Path,
    app_id: u32,
) -> SteamCompatToolConfigInspection {
    let config_path = steam_root.join("config").join("config.vdf");
    let root = match fs::canonicalize(steam_root) {
        Ok(root) => root,
        Err(error) => {
            return inspection_error(config_path, status_for_io(&error), error.to_string());
        }
    };
    match fs::metadata(&root) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => {
            return inspection_error(
                config_path,
                SteamCompatToolConfigStatus::UnsafePath,
                "Steam root is not a directory".into(),
            );
        }
        Err(error) => {
            return inspection_error(config_path, status_for_io(&error), error.to_string());
        }
    }

    let config_dir = root.join("config");
    match fs::symlink_metadata(&config_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return inspection_error(
                config_path,
                SteamCompatToolConfigStatus::UnsafePath,
                "Steam config path is not a regular directory".into(),
            );
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return inspection_error(
                config_path,
                SteamCompatToolConfigStatus::Missing,
                "Steam config directory was not found".into(),
            );
        }
        Err(error) => {
            return inspection_error(config_path, status_for_io(&error), error.to_string());
        }
    }

    match fs::symlink_metadata(&config_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return inspection_error(
                config_path,
                SteamCompatToolConfigStatus::UnsafePath,
                "config.vdf is not a regular file".into(),
            );
        }
        Ok(metadata) if metadata.len() > MAX_CONFIG_BYTES => {
            return inspection_error(
                config_path,
                SteamCompatToolConfigStatus::TooLarge,
                format!("config.vdf exceeds the {MAX_CONFIG_BYTES}-byte read limit"),
            );
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return inspection_error(
                config_path,
                SteamCompatToolConfigStatus::Missing,
                "Steam config.vdf was not found".into(),
            );
        }
        Err(error) => {
            return inspection_error(config_path, status_for_io(&error), error.to_string());
        }
    }

    let mut file = match File::open(&config_path) {
        Ok(file) => file,
        Err(error) => {
            return inspection_error(config_path, status_for_io(&error), error.to_string());
        }
    };
    let mut bytes = Vec::new();
    if let Err(error) = file
        .by_ref()
        .take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
    {
        return inspection_error(config_path, status_for_io(&error), error.to_string());
    }
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return inspection_error(
            config_path,
            SteamCompatToolConfigStatus::TooLarge,
            format!("config.vdf exceeds the {MAX_CONFIG_BYTES}-byte read limit"),
        );
    }
    let input = match std::str::from_utf8(&bytes) {
        Ok(input) => input,
        Err(error) => {
            return inspection_error(
                config_path,
                SteamCompatToolConfigStatus::Invalid,
                format!("config.vdf is not valid UTF-8: {error}"),
            );
        }
    };

    match parse_steam_compat_tool_mappings(input, app_id) {
        Ok(mappings) => SteamCompatToolConfigInspection {
            status: SteamCompatToolConfigStatus::Read,
            config_path,
            mappings: Some(mappings),
            detail: None,
        },
        Err(error) => inspection_error(
            config_path,
            SteamCompatToolConfigStatus::Invalid,
            error.to_string(),
        ),
    }
}

fn status_for_io(error: &io::Error) -> SteamCompatToolConfigStatus {
    if error.kind() == io::ErrorKind::PermissionDenied {
        SteamCompatToolConfigStatus::Unreadable
    } else if error.kind() == io::ErrorKind::NotFound {
        SteamCompatToolConfigStatus::Missing
    } else {
        SteamCompatToolConfigStatus::Invalid
    }
}

fn inspection_error(
    config_path: PathBuf,
    status: SteamCompatToolConfigStatus,
    detail: String,
) -> SteamCompatToolConfigInspection {
    SteamCompatToolConfigInspection {
        status,
        config_path,
        mappings: None,
        detail: Some(detail),
    }
}

fn required_object<'a>(
    entries: &'a [(String, KeyValuesValue)],
    key: &'static str,
) -> Result<&'a [(String, KeyValuesValue)], SteamCompatToolConfigParseError> {
    let value =
        unique_value(entries, key)?.ok_or(SteamCompatToolConfigParseError::SectionMissing(key))?;
    match value {
        KeyValuesValue::Object(entries) => Ok(entries),
        KeyValuesValue::Scalar(_) => Err(SteamCompatToolConfigParseError::ExpectedObject(key)),
    }
}

fn parse_mapping(
    entries: &[(String, KeyValuesValue)],
    key: &str,
    app_id: u32,
) -> Result<Option<SteamCompatToolMapping>, SteamCompatToolConfigParseError> {
    let Some(value) = unique_value(entries, key)? else {
        return Ok(None);
    };
    let fields = match value {
        KeyValuesValue::Object(fields) => fields,
        KeyValuesValue::Scalar(_) => {
            return Err(SteamCompatToolConfigParseError::ExpectedObject(
                "app mapping",
            ));
        }
    };
    Ok(Some(SteamCompatToolMapping {
        app_id,
        internal_name: optional_scalar(fields, "name")?,
        config: optional_scalar(fields, "config")?,
        priority: optional_scalar(fields, "priority")?,
    }))
}

fn optional_scalar(
    entries: &[(String, KeyValuesValue)],
    key: &'static str,
) -> Result<Option<String>, SteamCompatToolConfigParseError> {
    match unique_value(entries, key)? {
        Some(KeyValuesValue::Scalar(value)) => Ok(Some(value.clone())),
        Some(KeyValuesValue::Object(_)) => {
            Err(SteamCompatToolConfigParseError::ExpectedScalar(key))
        }
        None => Ok(None),
    }
}

fn unique_value<'a>(
    entries: &'a [(String, KeyValuesValue)],
    key: &str,
) -> Result<Option<&'a KeyValuesValue>, SteamCompatToolConfigParseError> {
    let mut matching = entries
        .iter()
        .filter(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value);
    let first = matching.next();
    if matching.next().is_some() {
        return Err(SteamCompatToolConfigParseError::DuplicateKey(
            key.to_owned(),
        ));
    }
    Ok(first)
}

#[cfg(test)]
mod tests {
    use super::{
        inspect_steam_compat_tool_config, parse_steam_compat_tool_mappings,
        SteamCompatToolConfigStatus, SteamCompatToolMapping,
    };
    use std::fs;
    use std::path::PathBuf;

    const APP_ID: u32 = 4162040;
    const PER_GAME_AND_GLOBAL: &str =
        include_str!("../tests/fixtures/compat_tools/per_game_and_global.vdf");
    const GLOBAL_ONLY: &str = include_str!("../tests/fixtures/compat_tools/global_only.vdf");
    const MALFORMED: &str = include_str!("../tests/fixtures/compat_tools/malformed.vdf");

    fn document(mapping: &str) -> String {
        format!(
            r#""InstallConfigStore" {{ "Software" {{ "Valve" {{ "Steam" {{ "CompatToolMapping" {{ {mapping} }} }} }} }} }}"#
        )
    }

    #[test]
    fn parses_explicit_per_game_mapping_separately_from_global_default() {
        let mappings = parse_steam_compat_tool_mappings(PER_GAME_AND_GLOBAL, APP_ID).unwrap();
        assert_eq!(
            mappings.per_app,
            Some(SteamCompatToolMapping {
                app_id: APP_ID,
                internal_name: Some("GE-Proton-test".into()),
                config: Some(String::new()),
                priority: Some("250".into()),
            })
        );
        assert_eq!(
            mappings.global_default.unwrap().internal_name.as_deref(),
            Some("proton_experimental")
        );
    }

    #[test]
    fn missing_app_mapping_does_not_promote_global_default() {
        let mappings = parse_steam_compat_tool_mappings(GLOBAL_ONLY, APP_ID).unwrap();
        assert!(mappings.per_app.is_none());
        assert_eq!(
            mappings.global_default.unwrap().internal_name.as_deref(),
            Some("proton_experimental")
        );
    }

    #[test]
    fn malformed_mapping_and_unexpected_config_are_errors() {
        assert!(parse_steam_compat_tool_mappings(MALFORMED, APP_ID).is_err());
        assert!(parse_steam_compat_tool_mappings(
            &document(r#""4162040" "proton_experimental""#),
            APP_ID
        )
        .is_err());
        assert!(parse_steam_compat_tool_mappings(
            r#""InstallConfigStore" { "Software" { "Valve" { "Steam" { "Other" {} } } } }"#,
            APP_ID
        )
        .is_err());
    }

    #[test]
    fn inspection_reads_a_synthetic_config_without_writing() {
        let root = std::env::temp_dir().join(format!(
            "lxmi-steam-config-test-{}-{}",
            std::process::id(),
            APP_ID
        ));
        let config_dir = root.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.vdf"), document("")).unwrap();
        let before = fs::metadata(config_dir.join("config.vdf")).unwrap().len();
        let inspection = inspect_steam_compat_tool_config(&root, APP_ID);
        let after = fs::metadata(config_dir.join("config.vdf")).unwrap().len();
        assert_eq!(inspection.status, SteamCompatToolConfigStatus::Read);
        assert!(inspection.mappings.unwrap().per_app.is_none());
        assert_eq!(before, after);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn inspection_reports_missing_configuration_as_unknown_evidence() {
        let root = PathBuf::from("/tmp/lxmi-steam-config-does-not-exist");
        let inspection = inspect_steam_compat_tool_config(&root, APP_ID);
        assert_eq!(inspection.status, SteamCompatToolConfigStatus::Missing);
        assert!(inspection.mappings.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn inspection_rejects_symlinked_config_directory() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "lxmi-steam-config-symlink-test-{}-{}",
            std::process::id(),
            APP_ID
        ));
        let outside = root.with_extension("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::create_dir_all(&root).unwrap();
        symlink(&outside, root.join("config")).unwrap();

        let inspection = inspect_steam_compat_tool_config(&root, APP_ID);
        assert_eq!(inspection.status, SteamCompatToolConfigStatus::UnsafePath);
        assert!(inspection.mappings.is_none());

        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&outside).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn inspection_rejects_symlinked_config_file() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "lxmi-steam-config-file-symlink-test-{}-{}",
            std::process::id(),
            APP_ID
        ));
        let config_dir = root.join("config");
        let outside = root.with_extension("outside");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(&outside, document("")).unwrap();
        symlink(&outside, config_dir.join("config.vdf")).unwrap();

        let inspection = inspect_steam_compat_tool_config(&root, APP_ID);
        assert_eq!(inspection.status, SteamCompatToolConfigStatus::UnsafePath);
        assert!(inspection.mappings.is_none());

        fs::remove_dir_all(&root).unwrap();
        fs::remove_file(outside).unwrap();
    }
}
