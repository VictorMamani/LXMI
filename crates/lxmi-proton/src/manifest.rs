use lxmi_steam::{parse_key_values_document, KeyValuesValue, VdfParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityToolMetadata {
    pub internal_id: String,
    pub display_name: Option<String>,
    pub install_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolManifestMetadata {
    pub compatmanager_layer_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityMetadataError {
    pub message: String,
}

impl std::fmt::Display for CompatibilityMetadataError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CompatibilityMetadataError {}

impl From<VdfParseError> for CompatibilityMetadataError {
    fn from(error: VdfParseError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

pub fn parse_compatibility_tool_vdf(
    input: &str,
) -> Result<Vec<CompatibilityToolMetadata>, CompatibilityMetadataError> {
    let document = parse_key_values_document(input)?;
    let compatibility_tools = unique_field(&document, "compatibilitytools")?
        .ok_or_else(|| missing("compatibilitytools"))?;
    let compatibility_tools = as_object(compatibility_tools, "compatibilitytools")?;
    let compat_tools = unique_field(compatibility_tools, "compat_tools")?
        .ok_or_else(|| missing("compat_tools"))?;
    let compat_tools = as_object(compat_tools, "compat_tools")?;

    let mut tools = Vec::new();
    for (internal_id, value) in compat_tools {
        if internal_id.trim().is_empty() {
            return Err(missing("internal tool id"));
        }
        let fields = as_object(value, "compatibility tool entry")?;
        let install_path =
            scalar_field(fields, "install_path")?.ok_or_else(|| missing("install_path"))?;
        let display_name = scalar_field(fields, "display_name")?;
        tools.push(CompatibilityToolMetadata {
            internal_id: internal_id.clone(),
            display_name,
            install_path,
        });
    }

    if tools.is_empty() {
        return Err(missing("compatibility tool entry"));
    }
    Ok(tools)
}

pub fn parse_tool_manifest_vdf(
    input: &str,
) -> Result<ToolManifestMetadata, CompatibilityMetadataError> {
    let document = parse_key_values_document(input)?;
    let manifest = unique_field(&document, "manifest")?.ok_or_else(|| missing("manifest"))?;
    let manifest = as_object(manifest, "manifest")?;
    Ok(ToolManifestMetadata {
        compatmanager_layer_name: scalar_field(manifest, "compatmanager_layer_name")?,
    })
}

fn unique_field<'a>(
    entries: &'a [(String, KeyValuesValue)],
    key: &str,
) -> Result<Option<&'a KeyValuesValue>, CompatibilityMetadataError> {
    let mut matches = entries
        .iter()
        .filter(|(entry_key, _)| entry_key.eq_ignore_ascii_case(key));
    let first = matches.next().map(|(_, value)| value);
    if matches.next().is_some() {
        return Err(CompatibilityMetadataError {
            message: format!("duplicate `{key}` field"),
        });
    }
    Ok(first)
}

fn scalar_field(
    entries: &[(String, KeyValuesValue)],
    key: &str,
) -> Result<Option<String>, CompatibilityMetadataError> {
    let Some(value) = unique_field(entries, key)? else {
        return Ok(None);
    };
    match value {
        KeyValuesValue::Scalar(value) => Ok(Some(value.clone())),
        KeyValuesValue::Object(_) => Err(CompatibilityMetadataError {
            message: format!("`{key}` must be a scalar value"),
        }),
    }
}

fn as_object<'a>(
    value: &'a KeyValuesValue,
    key: &str,
) -> Result<&'a [(String, KeyValuesValue)], CompatibilityMetadataError> {
    match value {
        KeyValuesValue::Object(entries) => Ok(entries),
        KeyValuesValue::Scalar(_) => Err(CompatibilityMetadataError {
            message: format!("`{key}` must be an object"),
        }),
    }
}

fn missing(field: &str) -> CompatibilityMetadataError {
    CompatibilityMetadataError {
        message: format!("required `{field}` field is missing"),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_compatibility_tool_vdf, parse_tool_manifest_vdf};

    #[test]
    fn parses_custom_compatibility_tool_metadata_and_ignores_unknown_fields() {
        let input = include_str!("../tests/fixtures/proton/custom/compatibilitytool.vdf");
        let tools = parse_compatibility_tool_vdf(input).expect("synthetic metadata parses");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].internal_id, "GE-Proton-fixture");
        assert_eq!(tools[0].display_name.as_deref(), Some("GE-Proton fixture"));
        assert_eq!(tools[0].install_path, ".");
    }

    #[test]
    fn rejects_missing_and_duplicate_compatibility_fields() {
        assert!(parse_compatibility_tool_vdf(
            r#""compatibilitytools" { "compat_tools" { "tool" { "display_name" "x" } } }"#
        )
        .is_err());
        assert!(parse_compatibility_tool_vdf(
            r#""compatibilitytools" { "compatibilitytools" { } }"#
        )
        .is_err());
        assert!(parse_compatibility_tool_vdf(
            r#""compatibilitytools" { "compat_tools" { "tool" { "install_path" "." "install_path" ".." } } }"#
        )
        .is_err());
    }

    #[test]
    fn parses_proton_tool_manifest_layer_marker() {
        let input = include_str!("../tests/fixtures/proton/steam-proton/toolmanifest.vdf");
        let metadata = parse_tool_manifest_vdf(input).expect("synthetic manifest parses");
        assert_eq!(metadata.compatmanager_layer_name.as_deref(), Some("proton"));
    }

    #[test]
    fn parses_linux_runtime_tool_manifest_as_a_different_layer() {
        let input = include_str!("../tests/fixtures/proton/linux-runtime/toolmanifest.vdf");
        let metadata = parse_tool_manifest_vdf(input).expect("synthetic manifest parses");
        assert_eq!(
            metadata.compatmanager_layer_name.as_deref(),
            Some("container-runtime")
        );
    }

    #[test]
    fn rejects_truncated_or_wrong_root_tool_manifests() {
        assert!(parse_tool_manifest_vdf(r#""manifest" { "#).is_err());
        assert!(parse_tool_manifest_vdf(r#""other" { }"#).is_err());
    }
}
