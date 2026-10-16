//! Read-only semantic diagnostics. Findings never authorize prefix entry or a loader.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const LIMIT: usize = 8 * 1024 * 1024;
type Values = BTreeMap<String, String>;
type Registry = BTreeMap<String, Values>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryDiagnosisError {
    TooLarge,
    IntegrityMismatch,
    UnsupportedFormat,
    DuplicateSection,
    DuplicateValue,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RegistryChangeDiagnosis {
    pub content_changed: bool,
    pub changed_sections: usize,
    pub font_sections: usize,
    pub font_values_added: usize,
    pub font_values_removed: usize,
    pub font_values_modified: usize,
    pub run_host_to_native_font_paths: usize,
    pub native_to_run_host_font_paths: usize,
    pub rewritten_font_filename_matches: usize,
    pub device_container_id_sections: usize,
    pub empty_network_cards_key_removed: bool,
    pub unknown_sections: usize,
    /// Diagnosis is not permission; the original UnexpectedMutation remains unchanged.
    pub execution_authorized: bool,
}

fn parse(input: &[u8]) -> Result<Registry, RegistryDiagnosisError> {
    use RegistryDiagnosisError::*;
    if input.len() > LIMIT {
        return Err(TooLarge);
    }
    let text = std::str::from_utf8(input).map_err(|_| UnsupportedFormat)?;
    let mut lines = text.lines();
    if lines.next() != Some("WINE REGISTRY Version 2") {
        return Err(UnsupportedFormat);
    }
    let mut out = Registry::new();
    out.insert("\0header".into(), Values::new());
    let mut section: Option<String> = None;
    let mut pending = String::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with(";;") || line.starts_with("#time=") {
            continue;
        }
        if line == "#arch=win64" || line == "#arch=win32" {
            if out
                .get_mut("\0header")
                .ok_or(UnsupportedFormat)?
                .insert("arch".into(), line.into())
                .is_some()
            {
                return Err(UnsupportedFormat);
            }
            continue;
        }
        if line == "#link" && pending.is_empty() {
            let key = section.as_ref().ok_or(UnsupportedFormat)?;
            if out
                .get_mut(key)
                .ok_or(UnsupportedFormat)?
                .insert("\0link".into(), "true".into())
                .is_some()
            {
                return Err(UnsupportedFormat);
            }
            continue;
        }
        if line.starts_with('[') && pending.is_empty() {
            let end = line.rfind(']').ok_or(UnsupportedFormat)?;
            let suffix = line[end + 1..].trim();
            if !suffix.is_empty() && !suffix.bytes().all(|b| b.is_ascii_digit()) {
                return Err(UnsupportedFormat);
            }
            let key = line[1..end].replace("\\\\", "\\");
            if key.is_empty() || key.contains('\0') {
                return Err(UnsupportedFormat);
            }
            if out.insert(key.clone(), Values::new()).is_some() {
                return Err(DuplicateSection);
            }
            section = Some(key);
            continue;
        }
        if line.starts_with('#') {
            return Err(UnsupportedFormat);
        }
        pending.push_str(line);
        if pending.ends_with('\\') {
            pending.pop();
            continue;
        }
        let key = section.as_ref().ok_or(UnsupportedFormat)?;
        // '=' inside quoted names is not a delimiter.
        let mut quoted = false;
        let mut escaped = false;
        let delimiter = pending
            .char_indices()
            .find_map(|(i, c)| {
                if escaped {
                    escaped = false;
                    return None;
                }
                if c == '\\' {
                    escaped = true;
                    return None;
                }
                if c == '"' {
                    quoted = !quoted;
                }
                (c == '=' && !quoted).then_some(i)
            })
            .ok_or(UnsupportedFormat)?;
        let name = &pending[..delimiter];
        if !(name == "@" || (name.starts_with('"') && name.ends_with('"'))) {
            return Err(UnsupportedFormat);
        }
        let value = &pending[delimiter + 1..];
        if value.is_empty() {
            return Err(UnsupportedFormat);
        }
        if out
            .get_mut(key)
            .ok_or(UnsupportedFormat)?
            .insert(name.into(), value.into())
            .is_some()
        {
            return Err(DuplicateValue);
        }
        pending.clear();
    }
    if !pending.is_empty() {
        return Err(UnsupportedFormat);
    }
    Ok(out)
}

fn string_value(value: &str) -> Option<String> {
    let value = value.strip_prefix('"')?.strip_suffix('"')?;
    Some(value.replace("\\\\", "\\").replace("\\\"", "\""))
}
fn is_fonts(key: &str) -> bool {
    matches!(
        key,
        "Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts"
            | "Software\\Microsoft\\Windows\\CurrentVersion\\Fonts"
            | "Software\\Wine\\Fonts\\External Fonts"
    )
}
/// Exact hashes of the three selected font tables; never returns registry values.
/// A stable container fixture is not evidence for a different initial font state.
pub fn selected_font_state_fingerprints(
    registry: &BTreeMap<String, Vec<u8>>,
) -> Result<BTreeMap<String, String>, RegistryDiagnosisError> {
    let mut out = BTreeMap::new();
    for (hive, keys) in [
        (
            "system.reg",
            vec![
                "Software\\Microsoft\\Windows NT\\CurrentVersion\\Fonts",
                "Software\\Microsoft\\Windows\\CurrentVersion\\Fonts",
            ],
        ),
        ("user.reg", vec!["Software\\Wine\\Fonts\\External Fonts"]),
    ] {
        let parsed = parse(
            registry
                .get(hive)
                .ok_or(RegistryDiagnosisError::UnsupportedFormat)?,
        )?;
        for key in keys {
            let values = parsed
                .get(key)
                .ok_or(RegistryDiagnosisError::UnsupportedFormat)?;
            out.insert(
                format!("{hive}:{key}"),
                format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(values).expect("string map"))
                ),
            );
        }
    }
    Ok(out)
}
fn is_device(key: &str) -> bool {
    let parts: Vec<_> = key.split('\\').collect();
    parts.len() == 6
        && parts[0] == "System"
        && parts[1] == "ControlSet001"
        && parts[2] == "Enum"
        && matches!(parts[3], "HID" | "WINEBUS")
        && !parts[4].is_empty()
        && !parts[5].is_empty()
}
/// Expected hashes must come from the reviewed before/after receipt, not a new baseline.
/// Values and arbitrary registry key names are never returned or logged.
pub fn diagnose_registry_change(
    before: &[u8],
    after: &[u8],
    expected_before_sha256: &str,
    expected_after_sha256: &str,
) -> Result<RegistryChangeDiagnosis, RegistryDiagnosisError> {
    if before.len() > LIMIT || after.len() > LIMIT {
        return Err(RegistryDiagnosisError::TooLarge);
    }
    if format!("{:x}", Sha256::digest(before)) != expected_before_sha256
        || format!("{:x}", Sha256::digest(after)) != expected_after_sha256
    {
        return Err(RegistryDiagnosisError::IntegrityMismatch);
    }
    let old = parse(before)?;
    let new = parse(after)?;
    let mut report = RegistryChangeDiagnosis {
        content_changed: before != after,
        ..Default::default()
    };
    let keys: BTreeSet<_> = old.keys().chain(new.keys()).collect();
    for key in keys {
        let a = old.get(key);
        let b = new.get(key);
        if a == b {
            continue;
        }
        report.changed_sections += 1;
        if is_fonts(key) {
            report.font_sections += 1;
            let empty = Values::new();
            let a = a.unwrap_or(&empty);
            let b = b.unwrap_or(&empty);
            for name in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                match (a.get(name), b.get(name)) {
                    (None, Some(_)) => report.font_values_added += 1,
                    (Some(_), None) => report.font_values_removed += 1,
                    (Some(x), Some(y)) if x != y => {
                        report.font_values_modified += 1;
                        if let (Some(x), Some(y)) = (string_value(x), string_value(y)) {
                            if x.starts_with("Z:\\run\\host\\")
                                && y.starts_with("Z:\\usr\\share\\fonts\\")
                            {
                                report.run_host_to_native_font_paths += 1;
                                if x.rsplit('\\').next() == y.rsplit('\\').next() {
                                    report.rewritten_font_filename_matches += 1;
                                }
                            } else if x.starts_with("Z:\\usr\\share\\fonts\\")
                                && y.starts_with("Z:\\run\\host\\")
                            {
                                report.native_to_run_host_font_paths += 1;
                                if x.rsplit('\\').next() == y.rsplit('\\').next() {
                                    report.rewritten_font_filename_matches += 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else if is_device(key) && a.is_some() && b.is_some() {
            let a = a.unwrap();
            let b = b.unwrap();
            let changed: BTreeSet<_> = a
                .keys()
                .chain(b.keys())
                .filter(|name| a.get(*name) != b.get(*name))
                .collect();
            if changed.len() == 1 && changed.contains(&"\"ContainerId\"".to_string()) {
                report.device_container_id_sections += 1;
            } else {
                report.unknown_sections += 1;
            }
        } else if key == "Software\\Microsoft\\Windows NT\\CurrentVersion\\NetworkCards"
            && a.is_some_and(Values::is_empty)
            && b.is_none()
        {
            report.empty_network_cards_key_removed = true;
        } else {
            report.unknown_sections += 1;
        }
    }
    Ok(report)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureRegistrySectionMutation {
    pub section: String,
    pub changed_fields: Vec<String>,
    pub before_sha256: Option<String>,
    pub after_sha256: Option<String>,
    pub classification: super::PrefixMutationClassification,
    pub evidence: String,
}
/// Fixture diagnostics only: concrete section/field names, hashes instead of values.
/// Unknown fields within known sections remain Unknown; no section-wide exclusion.
pub fn diagnose_fixture_registry_sections(
    before: &[u8],
    after: &[u8],
) -> Result<Vec<FixtureRegistrySectionMutation>, RegistryDiagnosisError> {
    use super::PrefixMutationClassification as C;
    let old = parse(before)?;
    let new = parse(after)?;
    let mut report = Vec::new();
    for key in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
        let a = old.get(key);
        let b = new.get(key);
        if a == b {
            continue;
        }
        let empty = Values::new();
        let fields: Vec<String> = a
            .unwrap_or(&empty)
            .keys()
            .chain(b.unwrap_or(&empty).keys())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|n| a.and_then(|v| v.get(*n)) != b.and_then(|v| v.get(*n)))
            .cloned()
            .collect();
        let (classification, evidence) = if is_fonts(key)
            && !fields.is_empty()
            && fields.iter().all(|field| {
                [a.and_then(|v| v.get(field)), b.and_then(|v| v.get(field))]
                    .into_iter()
                    .flatten()
                    .all(|v| supported_font_path(v))
            }) {
            (C::EnvironmentSpecific, "Wine win32u/font.c and pressure-vessel host font mounts; verify against controlled fixture counts")
        } else if is_device(key)
            && a.is_some()
            && b.is_some()
            && fields == ["\"ContainerId\""]
            && [a.unwrap(), b.unwrap()].iter().all(|v| {
                v.get("\"ContainerId\"")
                    .and_then(|s| string_value(s))
                    .is_some_and(|s| valid_guid(&s))
            })
        {
            (C::VolatileExpected, "Wine adea6cba winebus.sys/main.c make_unique_container_id uses QueryPerformanceCounter; ntoskrnl PnP writes SPDRP_BASE_CONTAINERID. Fixture scope only; no functional harmlessness claim")
        } else if a.is_none()
            && b.is_some_and(Values::is_empty)
            && matches!(
                key.as_str(),
                "Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\RunServices"
                    | "Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\RunServicesOnce"
            )
        {
            (C::StableExpected, "Wine adea6cba wineboot ProcessRunKeys/process_run_key creates the exact empty 32-bit keys; only absent-to-empty creation, never values, is classified")
        } else if key == "Software\\Microsoft\\Windows NT\\CurrentVersion\\NetworkCards"
            && a.is_some_and(Values::is_empty)
            && b.is_none()
        {
            (C::StableExpected, "DERIVED: Wine adea6cba NDIS add_key creates volatile children; server save_subkeys omits an empty parent with children then skips volatile children. Saved-hive omission does not prove live registry removal. Only this exact empty-key removal is classified")
        } else {
            (C::Unknown, "No reviewed per-field mutation rule; exact section retained and blocks qualification")
        };
        let hash = |v: Option<&Values>| {
            v.map(|v| {
                format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(v).expect("string map"))
                )
            })
        };
        report.push(FixtureRegistrySectionMutation {
            section: key.clone(),
            changed_fields: fields,
            before_sha256: hash(a),
            after_sha256: hash(b),
            classification,
            evidence: evidence.into(),
        });
    }
    Ok(report)
}
fn supported_font_path(value: &str) -> bool {
    string_value(value).is_some_and(|path| {
        // Wine/fontconfig serialized paths can contain doubled Windows separators.
        let path = path.replace("\\\\", "\\");
        (path.starts_with("Z:\\run\\host\\fonts\\")
            || path.starts_with("Z:\\run\\host\\local-fonts\\")
            || path.starts_with("Z:\\usr\\share\\fonts\\")
            || path.starts_with("Z:\\usr\\local\\share\\fonts\\")
            || path.starts_with("Z:\\usr\\share\\texmf\\fonts\\"))
            && path.rsplit('.').next().is_some_and(|ext| {
                matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "ttf" | "ttc" | "otf" | "fon" | "pfb" | "pfa"
                )
            })
    })
}
fn valid_guid(s: &str) -> bool {
    let Some(s) = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        return false;
    };
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn run(a: &str, b: &str) -> Result<RegistryChangeDiagnosis, RegistryDiagnosisError> {
        diagnose_registry_change(
            a.as_bytes(),
            b.as_bytes(),
            &format!("{:x}", Sha256::digest(a)),
            &format!("{:x}", Sha256::digest(b)),
        )
    }

    #[test]
    fn exact_tex_font_paths_and_doubled_separators_are_environment_specific() {
        use super::super::PrefixMutationClassification as C;
        let empty = b"WINE REGISTRY Version 2\n";
        let changed = br#"WINE REGISTRY Version 2
[Software\\Wine\\Fonts\\External Fonts] 1
"TeX"="Z:\\\\usr\\\\share\\\\texmf\\\\fonts\\\\opentype\\\\fixture.otf"
"Regular"="Z:\\usr\\share\\fonts\\fixture.ttf"
"#;
        assert_eq!(
            diagnose_fixture_registry_sections(empty, changed).unwrap()[0].classification,
            C::EnvironmentSpecific
        );
        let unknown = String::from_utf8(changed.to_vec())
            .unwrap()
            .replace("texmf", "unrelated");
        assert_eq!(
            diagnose_fixture_registry_sections(empty, unknown.as_bytes()).unwrap()[0]
                .classification,
            C::Unknown
        );
    }
    #[test]
    fn empty_run_services_initialization_does_not_allow_commands_or_unknown_font_fields() {
        let empty = b"WINE REGISTRY Version 2\n";
        let created=b"WINE REGISTRY Version 2\n[Software\\\\Wow6432Node\\\\Microsoft\\\\Windows\\\\CurrentVersion\\\\RunServices] 1\n";
        assert_eq!(
            diagnose_fixture_registry_sections(empty, created).unwrap()[0].classification,
            super::super::PrefixMutationClassification::StableExpected
        );
        let nonempty = [created.as_slice(), b"\"command\"=\"never execute\"\n"].concat();
        assert_eq!(
            diagnose_fixture_registry_sections(empty, &nonempty).unwrap()[0].classification,
            super::super::PrefixMutationClassification::Unknown
        );
        let font=b"WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts] 1\n\"Unexpected\"=dword:00000001\n";
        assert_eq!(
            diagnose_fixture_registry_sections(empty, font).unwrap()[0].classification,
            super::super::PrefixMutationClassification::Unknown
        );
    }
    #[test]
    fn nonempty_networkcards_removal_is_not_the_empty_serialization_rule() {
        let before=b"WINE REGISTRY Version 2\n[Software\\\\Microsoft\\\\Windows NT\\\\CurrentVersion\\\\NetworkCards] 1\n\"data\"=\"important\"\n";
        assert_eq!(
            diagnose_fixture_registry_sections(before, b"WINE REGISTRY Version 2\n").unwrap()[0]
                .classification,
            super::super::PrefixMutationClassification::Unknown
        );
    }
    #[test]
    fn exact_device_guid_change_is_volatile_but_additional_field_stays_unknown() {
        let a="WINE REGISTRY Version 2\n[System\\\\ControlSet001\\\\Enum\\\\WINEBUS\\\\id\\\\instance] 1\n\"ContainerId\"=\"{12345678-1234-1234-1234-123456789012}\"\n";
        let b = a.replace("123456789012", "123456789013");
        let r = diagnose_fixture_registry_sections(a.as_bytes(), b.as_bytes()).unwrap();
        assert_eq!(
            r[0].classification,
            super::super::PrefixMutationClassification::VolatileExpected
        );
        assert!(r[0].before_sha256.is_some());
        let b = format!("{b}\"NewField\"=\"unexpected\"\n");
        assert_eq!(
            diagnose_fixture_registry_sections(a.as_bytes(), b.as_bytes()).unwrap()[0]
                .classification,
            super::super::PrefixMutationClassification::Unknown
        );
    }
    #[test]
    fn network_cards_empty_saved_key_omission_is_narrow_and_evidenced() {
        let a="WINE REGISTRY Version 2\n[Software\\\\Microsoft\\\\Windows NT\\\\CurrentVersion\\\\NetworkCards] 1\n";
        let r =
            diagnose_fixture_registry_sections(a.as_bytes(), b"WINE REGISTRY Version 2\n").unwrap();
        assert!(r[0].section.ends_with("NetworkCards"));
        assert_eq!(
            r[0].classification,
            super::super::PrefixMutationClassification::StableExpected
        );
        assert!(r[0].after_sha256.is_none());
    }
    #[test]
    fn font_mount_rewrite_is_diagnostic_never_authorization() {
        let a = "WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts] 1\n\"Font\"=\"Z:\\\\run\\\\host\\\\fonts\\\\font.ttf\"\n";
        let b = a.replace("run\\\\host\\\\fonts", "usr\\\\share\\\\fonts");
        let r = run(a, &b).unwrap();
        assert_eq!(r.font_values_modified, 1);
        assert_eq!(r.run_host_to_native_font_paths, 1);
        assert_eq!(r.rewritten_font_filename_matches, 1);
        assert!(!r.execution_authorized);
    }
    #[test]
    fn timestamps_are_distinct_from_semantic_changes() {
        let r = run(
            "WINE REGISTRY Version 2\n[A] 1\n#time=123\n\"x\"=dword:00000001\n",
            "WINE REGISTRY Version 2\n[A] 2\n#time=456\n\"x\"=dword:00000001\n",
        )
        .unwrap();
        assert!(r.content_changed);
        assert_eq!(r.changed_sections, 0);
        assert!(!r.execution_authorized);
    }
    #[test]
    fn reverse_mount_transition_remains_diagnostic_only() {
        let a = "WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts]\n\"Font\"=\"Z:\\\\usr\\\\share\\\\fonts\\\\font.ttf\"\n";
        let b = a.replace("usr\\\\share\\\\fonts", "run\\\\host\\\\fonts");
        let r = run(a, &b).unwrap();
        assert_eq!(r.native_to_run_host_font_paths, 1);
        assert_eq!(r.run_host_to_native_font_paths, 0);
        assert_eq!(r.rewritten_font_filename_matches, 1);
        assert!(!r.execution_authorized);
    }
    #[test]
    fn malformed_duplicate_and_incomplete_inputs_fail_closed() {
        for (s, e) in [
            ("bad", RegistryDiagnosisError::UnsupportedFormat),
            (
                "WINE REGISTRY Version 2\n[A]\n[A]\n",
                RegistryDiagnosisError::DuplicateSection,
            ),
            (
                "WINE REGISTRY Version 2\n[A]\n\"x\"=1\n\"x\"=2\n",
                RegistryDiagnosisError::DuplicateValue,
            ),
            (
                "WINE REGISTRY Version 2\n[A]\n\"x\"=hex:01,\\\n",
                RegistryDiagnosisError::UnsupportedFormat,
            ),
        ] {
            assert_eq!(run(s, s).unwrap_err(), e);
        }
    }
    #[test]
    fn hash_mismatch_rejects_changed_source() {
        assert_eq!(
            diagnose_registry_change(b"a", b"b", "bad", "bad").unwrap_err(),
            RegistryDiagnosisError::IntegrityMismatch
        );
    }
    #[test]
    fn container_id_only_is_distinct_from_other_device_changes() {
        let a = "WINE REGISTRY Version 2\n[System\\\\ControlSet001\\\\Enum\\\\HID\\\\Fixture\\\\Instance] 1\n\"ContainerId\"=\"old\"\n";
        let b = a.replace("old", "new");
        assert_eq!(run(a, &b).unwrap().device_container_id_sections, 1);
        assert_eq!(
            run(a, &(b + "\"Service\"=\"changed\"\n"))
                .unwrap()
                .unknown_sections,
            1
        );
    }
    #[test]
    fn unknown_values_never_leak_to_report() {
        let a = "WINE REGISTRY Version 2\n[Private\\\\User]\n\"Password\"=\"sensitive-before\"\n";
        let b = a.replace("before", "after");
        let r = run(a, &b).unwrap();
        assert_eq!(r.unknown_sections, 1);
        let text = serde_json::to_string(&r).unwrap();
        assert!(!text.contains("sensitive"));
        assert!(!text.contains("Private"));
    }
    #[test]
    fn multiline_hex_and_equals_inside_value_names_parse() {
        let a = "WINE REGISTRY Version 2\n[A]\n\"x=y\"=hex:01,\\\n02\n";
        assert_eq!(run(a, a).unwrap().changed_sections, 0);
    }
    #[test]
    fn font_addition_removal_and_empty_key_removal_are_explicit() {
        let a="WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts]\n\"old\"=\"a\"\n[Software\\\\Microsoft\\\\Windows NT\\\\CurrentVersion\\\\NetworkCards]\n";
        let b="WINE REGISTRY Version 2\n[Software\\\\Wine\\\\Fonts\\\\External Fonts]\n\"new\"=\"b\"\n";
        let r = run(a, b).unwrap();
        assert_eq!(r.font_values_added, 1);
        assert_eq!(r.font_values_removed, 1);
        assert!(r.empty_network_cards_key_removed);
        assert!(!r.execution_authorized);
    }
    #[test]
    fn registry_link_marker_and_architecture_are_not_discarded() {
        let a = "WINE REGISTRY Version 2\n#arch=win64\n[A]\n#link\n@=\"target\"\n";
        assert_eq!(run(a, a).unwrap().changed_sections, 0);
        assert_eq!(
            run(a, &a.replace("#arch=win64", "#arch=win32"))
                .unwrap()
                .unknown_sections,
            1
        );
        assert_eq!(
            run(a, &a.replace("#link\n", "")).unwrap().unknown_sections,
            1
        );
    }
    #[test]
    fn oversized_data_is_rejected_before_hashing() {
        assert_eq!(
            diagnose_registry_change(&vec![0; LIMIT + 1], b"", "", "").unwrap_err(),
            RegistryDiagnosisError::TooLarge
        );
    }
    #[test]
    fn nonempty_network_key_removal_stays_unknown() {
        let a="WINE REGISTRY Version 2\n[Software\\\\Microsoft\\\\Windows NT\\\\CurrentVersion\\\\NetworkCards]\n\"x\"=dword:00000001\n";
        let r = run(a, "WINE REGISTRY Version 2\n").unwrap();
        assert!(!r.empty_network_cards_key_removed);
        assert_eq!(r.unknown_sections, 1);
        assert!(!r.execution_authorized);
    }
}
