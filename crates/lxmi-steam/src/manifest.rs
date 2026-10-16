use crate::vdf::{parse_key_values_document, KeyValuesValue, VdfParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamAppManifest {
    pub app_id: u32,
    pub name: String,
    pub install_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamManifestError {
    Document(VdfParseError),
    AppStateMissing,
    MissingField(&'static str),
    DuplicateField(&'static str),
    ExpectedScalar(&'static str),
    InvalidAppId,
    EmptyField(&'static str),
    InvalidInstallDirectory,
}

impl std::fmt::Display for SteamManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document(error) => write!(formatter, "invalid KeyValues document: {error}"),
            Self::AppStateMissing => write!(formatter, "AppState object is missing"),
            Self::MissingField(field) => write!(formatter, "required field {field} is missing"),
            Self::DuplicateField(field) => {
                write!(formatter, "field {field} appears more than once")
            }
            Self::ExpectedScalar(field) => {
                write!(formatter, "field {field} must be a scalar value")
            }
            Self::InvalidAppId => write!(formatter, "appid must be a positive 32-bit integer"),
            Self::EmptyField(field) => write!(formatter, "field {field} must not be empty"),
            Self::InvalidInstallDirectory => write!(
                formatter,
                "installdir must be a single relative directory name"
            ),
        }
    }
}

impl std::error::Error for SteamManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Document(error) => Some(error),
            _ => None,
        }
    }
}

pub fn parse_app_manifest(input: &str) -> Result<SteamAppManifest, SteamManifestError> {
    let document = parse_key_values_document(input).map_err(SteamManifestError::Document)?;
    let app_state = unique_field(&document, "AppState")?
        .and_then(|value| match value {
            KeyValuesValue::Object(fields) => Some(fields),
            KeyValuesValue::Scalar(_) => None,
        })
        .ok_or(SteamManifestError::AppStateMissing)?;

    let app_id_value = required_scalar(app_state, "appid")?;
    let app_id = app_id_value
        .parse::<u32>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or(SteamManifestError::InvalidAppId)?;

    let name = required_scalar(app_state, "name")?.trim().to_owned();
    if name.is_empty() {
        return Err(SteamManifestError::EmptyField("name"));
    }

    let install_dir = required_scalar(app_state, "installdir")?.trim().to_owned();
    if install_dir.is_empty() {
        return Err(SteamManifestError::EmptyField("installdir"));
    }
    if !is_single_relative_directory(&install_dir) {
        return Err(SteamManifestError::InvalidInstallDirectory);
    }

    Ok(SteamAppManifest {
        app_id,
        name,
        install_dir,
    })
}

fn required_scalar<'a>(
    entries: &'a [(String, KeyValuesValue)],
    field: &'static str,
) -> Result<&'a str, SteamManifestError> {
    let value = unique_field(entries, field)?.ok_or(SteamManifestError::MissingField(field))?;
    match value {
        KeyValuesValue::Scalar(value) => Ok(value),
        KeyValuesValue::Object(_) => Err(SteamManifestError::ExpectedScalar(field)),
    }
}

fn unique_field<'a>(
    entries: &'a [(String, KeyValuesValue)],
    field: &'static str,
) -> Result<Option<&'a KeyValuesValue>, SteamManifestError> {
    let mut matches = entries
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case(field))
        .map(|(_, value)| value);
    let first = matches.next();
    if matches.next().is_some() {
        return Err(SteamManifestError::DuplicateField(field));
    }
    Ok(first)
}

fn is_single_relative_directory(value: &str) -> bool {
    if value.contains('/') || value.contains('\\') || value.contains('\0') {
        return false;
    }

    let mut components = std::path::Path::new(value).components();
    matches!(components.next(), Some(std::path::Component::Normal(_)))
        && components.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::{parse_app_manifest, SteamManifestError};
    use lxmi_core::WUTHERING_WAVES_APP_ID;

    const VALID: &str = include_str!("../tests/fixtures/appmanifests/valid.acf");
    const MISSING_NAME: &str = include_str!("../tests/fixtures/appmanifests/missing_name.acf");
    const MALFORMED: &str = include_str!("../tests/fixtures/appmanifests/malformed.acf");
    const ADDITIONAL_FIELDS: &str =
        include_str!("../tests/fixtures/appmanifests/additional_fields.acf");

    #[test]
    fn parses_required_fields_from_synthetic_wuthering_waves_fixture() {
        let manifest = parse_app_manifest(VALID).expect("valid fixture should parse");
        assert_eq!(manifest.app_id, WUTHERING_WAVES_APP_ID);
        assert_eq!(manifest.name, "Wuthering Waves");
        assert_eq!(manifest.install_dir, "Wuthering Waves");
    }

    #[test]
    fn ignores_additional_nested_fields_and_handles_whitespace() {
        let manifest = parse_app_manifest(ADDITIONAL_FIELDS).expect("extra fields are supported");
        assert_eq!(manifest.app_id, 123456);
        assert_eq!(manifest.name, "Synthetic Example");
        assert_eq!(manifest.install_dir, "Example Folder");
    }

    #[test]
    fn reports_required_fields_that_are_missing() {
        assert_eq!(
            parse_app_manifest(MISSING_NAME),
            Err(SteamManifestError::MissingField("name"))
        );
        assert_eq!(
            parse_app_manifest(r#""AppState" { "name" "Game" "installdir" "Game" }"#),
            Err(SteamManifestError::MissingField("appid"))
        );
        assert_eq!(
            parse_app_manifest(r#""AppState" { "appid" "123" "name" "Game" }"#),
            Err(SteamManifestError::MissingField("installdir"))
        );
    }

    #[test]
    fn rejects_invalid_app_ids_and_duplicate_required_fields() {
        assert_eq!(
            parse_app_manifest(
                r#""AppState" { "appid" "nope" "name" "Game" "installdir" "Game" }"#
            ),
            Err(SteamManifestError::InvalidAppId)
        );
        assert_eq!(
            parse_app_manifest(
                r#""AppState" { "appid" "123" "name" "Game" "name" "Other" "installdir" "Game" }"#
            ),
            Err(SteamManifestError::DuplicateField("name"))
        );
    }

    #[test]
    fn rejects_install_directories_that_could_escape_common() {
        for install_dir in ["../outside", "/tmp/outside", "nested/game", "..\\\\outside"] {
            let input = format!(
                r#""AppState" {{ "appid" "123" "name" "Game" "installdir" "{install_dir}" }}"#
            );
            assert_eq!(
                parse_app_manifest(&input),
                Err(SteamManifestError::InvalidInstallDirectory)
            );
        }
    }

    #[test]
    fn reports_truncated_manifests() {
        assert!(matches!(
            parse_app_manifest(MALFORMED),
            Err(SteamManifestError::Document(_))
        ));
    }
}
